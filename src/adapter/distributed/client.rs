use crate::{
    adapter::{
        CommandProcessor, DisputeIndexCallback, EngineContext, JournalTransactionLookup,
        PaymentEngine,
    },
    domain::{
        AccountState, ActiveAccountState, CommandMetadata, PaymentError, TransactionTypeCommand,
    },
    port::{DisputeIndex, Engine, Journal},
};
use async_trait::async_trait;
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use std::sync::Arc;

/// Messages that can be sent to a ClientActor
pub enum ClientActorMessage {
    ProcessCommand(
        TransactionTypeCommand,
        CommandMetadata,
        RpcReplyPort<Result<(), PaymentError>>,
    ),
    GetState(RpcReplyPort<AccountState>),
}

impl ractor::Message for ClientActorMessage {}

pub struct ClientActorArguments {
    pub client_id: u16,
    pub journal: Arc<dyn Journal + Send + Sync>,
    pub dispute_index: Arc<dyn DisputeIndex>,
}

pub struct ClientActorState {
    pub client_id: u16,
    pub account_state: AccountState,
    pub engine: Arc<dyn Engine<Context = EngineContext> + Send + Sync>,
    pub journal: Arc<dyn Journal + Send + Sync>,
    /// Last applied sequence number (global journal sequence, not per-client)
    /// Used to guarantee events are applied in order: seq[n] > seq[n-1]
    /// Also enables idempotent handling of Kafka at-least-once duplicates
    pub last_sequence: u64,
}

/// ClientActor manages a single client's account
/// Each client gets their own actor instance with isolated state
pub struct ClientActor;

#[async_trait]
impl Actor for ClientActor {
    type Msg = ClientActorMessage;
    type State = ClientActorState;
    type Arguments = ClientActorArguments;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!(
            "ClientActor starting for client {} (Akka-style)",
            args.client_id
        );

        let lookup = Arc::new(JournalTransactionLookup::new(
            args.journal.clone(),
            args.dispute_index.clone(),
        ));
        let processor = Arc::new(CommandProcessor::new(lookup));

        // Register DisputeIndexCallback to maintain infrastructure index via callbacks
        let dispute_callback = Arc::new(DisputeIndexCallback::new(args.dispute_index.clone()));
        let engine = Arc::new(PaymentEngine::new(processor).with_callback(dispute_callback));

        // TODO: This should be loaded from snapshot or database ->
        // If snapshot, then should do snapshot + apply pending events from journal
        let account_state = AccountState::Active(ActiveAccountState {
            available: 0.0,
            held: 0.0,
            total: 0.0,
            last_activity: chrono::Utc::now(),
        });

        Ok(ClientActorState {
            client_id: args.client_id,
            account_state,
            engine,
            journal: args.journal,
            last_sequence: 0, // Start from 0, first event will be sequence 1
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ClientActorMessage::ProcessCommand(command, metadata, reply) => {
                // CRITICAL: This actor provides ordering guarantees (infrastructure concern)!
                // We have &mut state, which means:
                // 1. Only ONE message processes at a time for this client
                // 2. Validation + persistence + state update happen atomically
                // 3. Events are applied in strict sequence order
                //
                // Flow: validate → persist → verify sequence → update state
                // If validation fails: state unchanged, nothing persisted ✅
                // If persistence fails: state unchanged ✅
                // If sequence is wrong: PANIC (infrastructure bug) ✅
                // If success: state updated atomically ✅

                let context = EngineContext {
                    journal: state.journal.clone(),
                    current_state: state.account_state.clone(),
                };

                match state
                    .engine
                    .process_command(command, metadata, &context)
                    .await
                {
                    Ok((envelope, new_state)) => {
                        // INFRASTRUCTURE GUARANTEE: Verify event ordering
                        // Sequence numbers are global (shared across all clients in journal),
                        // so we verify monotonic ordering for this client's events.
                        //
                        // Cases:
                        // 1. seq > last_sequence → Apply (normal case)
                        // 2. seq == last_sequence → Skip (Kafka at-least-once duplicate)
                        // 3. seq < last_sequence → PANIC (ordering violation)
                        //
                        // Example with Kafka at-least-once:
                        //   Process seq 5 → state.last_sequence = 5
                        //   Kafka redelivers → seq 5 again → SKIP (idempotent)
                        //   Process seq 7 → state.last_sequence = 7 (skip 6, other client)

                        if envelope.sequence_nr < state.last_sequence {
                            panic!(
                                "CRITICAL: Event ordering violation for client {}! \
                                 Last sequence was {}, got {}. This indicates a bug in \
                                 the infrastructure (out-of-order delivery).",
                                state.client_id, state.last_sequence, envelope.sequence_nr
                            );
                        }

                        if envelope.sequence_nr == state.last_sequence {
                            // Duplicate event (Kafka at-least-once) - already applied, skip
                            tracing::debug!(
                                "Client {} skipping duplicate event: seq={}",
                                state.client_id,
                                envelope.sequence_nr
                            );
                            let _ = reply.send(Ok(()));
                            return Ok(());
                        }

                        // Normal case: apply new event
                        state.account_state = new_state;
                        state.last_sequence = envelope.sequence_nr;

                        tracing::debug!(
                            "Client {} applied event: seq={} (previous={})",
                            state.client_id,
                            envelope.sequence_nr,
                            state.last_sequence - 1 // Show actual previous
                        );
                        let _ = reply.send(Ok(()));
                    }
                    Err(e) => {
                        // Validation or persistence failed - state unchanged
                        tracing::error!(
                            "Client {} failed to process command: {}",
                            state.client_id,
                            e
                        );
                        let _ = reply.send(Err(e));
                    }
                }
            }

            ClientActorMessage::GetState(reply) => {
                let _ = reply.send(state.account_state.clone());
            }
        }

        Ok(())
    }
}

/// Type alias for ClientActor reference
pub type ClientActorRef = ActorRef<ClientActorMessage>;
