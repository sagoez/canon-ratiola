use crate::adapter::{ClientActorArguments, ClientActorMessage};
use crate::domain::{
    AccountState, CommandMetadata, EngineError, PaymentError, TransactionTypeCommand,
};
use crate::port::{DisputeIndex, Journal};
use ractor::{Actor, ActorRef, rpc::CallResult};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

type ClientActorRef = ActorRef<ClientActorMessage>;

/// ClientRegistry uses ractor's global registry for distributed actor lookup
///
/// Instead of maintaining a local DashMap (split-brain risk), we rely on
/// ractor's built-in registry which has cluster-wide awareness via ActorRef::where_is().
///
/// This prevents split-brain: if two nodes try to spawn the same client actor,
/// only one succeeds (the named actor is a singleton in the cluster).
#[derive(Clone)]
pub struct ClientRegistry {
    /// Track which clients we've seen (for final output only, not for routing)
    /// This is local to this node/process - only used to know which client states to fetch
    processed_clients: Arc<Mutex<HashSet<u16>>>,
    /// Shared journal (passed to spawned actors)
    journal: Arc<dyn Journal + Send + Sync>,
    /// Shared dispute index (passed to spawned actors)
    dispute_index: Arc<dyn DisputeIndex>,
    /// Namespace prefix for actor names (for test isolation)
    namespace: String,
}

impl ClientRegistry {
    pub fn new(
        journal: Arc<dyn Journal + Send + Sync>,
        dispute_index: Arc<dyn DisputeIndex>,
    ) -> Self {
        Self {
            processed_clients: Arc::new(Mutex::new(HashSet::new())),
            journal,
            dispute_index,
            namespace: String::new(),
        }
    }

    /// Create a registry with a custom namespace for test isolation.
    ///
    /// ## Warning: This is NOT MEANT FOR PRODUCTION USE. Only for testing purposes.
    pub fn with_namespace(
        journal: Arc<dyn Journal + Send + Sync>,
        dispute_index: Arc<dyn DisputeIndex>,
        namespace: String,
    ) -> Self {
        Self {
            processed_clients: Arc::new(Mutex::new(HashSet::new())),
            journal,
            dispute_index,
            namespace,
        }
    }

    /// Get or spawn a client actor using ractor's global registry
    ///
    /// This is cluster-safe: ActorRef::where_is() checks the global registry,
    /// preventing split-brain issues where multiple nodes spawn the same client actor.
    pub async fn get_or_spawn(&self, client_id: u16) -> Result<ClientActorRef, PaymentError> {
        let actor_name = if self.namespace.is_empty() {
            format!("client-{}", client_id)
        } else {
            format!("{}-client-{}", self.namespace, client_id)
        };

        // Fast path: check ractor's global registry
        if let Some(actor_ref) = ActorRef::<ClientActorMessage>::where_is(actor_name.clone()) {
            return Ok(actor_ref);
        }

        // Slow path: spawn actor with global name
        // Race condition: another node might spawn it first and that's fine.
        // The registry ensures only one actor with this name exists cluster-wide
        let args = ClientActorArguments {
            client_id,
            journal: self.journal.clone(),
            dispute_index: self.dispute_index.clone(),
        };

        match Actor::spawn(Some(actor_name.clone()), super::client::ClientActor, args).await {
            Ok((actor_ref, _handle)) => Ok(actor_ref),
            Err(e) => {
                // Spawn failed - maybe another node just spawned it?
                // Try lookup one more time before giving up
                if let Some(actor_ref) = ActorRef::<ClientActorMessage>::where_is(actor_name) {
                    Ok(actor_ref)
                } else {
                    Err(PaymentError::Engine(EngineError::ValidationError(format!(
                        "Failed to spawn or find client actor: {:?}",
                        e
                    ))))
                }
            }
        }
    }

    /// Process a command for a client (get_or_spawn + send message)
    pub async fn process_command(
        &self,
        client_id: u16,
        command: TransactionTypeCommand,
        metadata: CommandMetadata,
    ) -> Result<(), PaymentError> {
        self.processed_clients.lock().unwrap().insert(client_id);

        let actor_ref = self.get_or_spawn(client_id).await?;

        match actor_ref
            .call(
                |reply| ClientActorMessage::ProcessCommand(command, metadata, reply),
                Some(std::time::Duration::from_millis(500)),
            )
            .await
        {
            Ok(CallResult::Success(Ok(()))) => Ok(()),
            Ok(CallResult::Success(Err(e))) => Err(e),
            Ok(CallResult::Timeout) => Err(PaymentError::Engine(EngineError::ValidationError(
                "Actor call timeout".to_string(),
            ))),
            Ok(CallResult::SenderError) => Err(PaymentError::Engine(EngineError::ValidationError(
                "Actor sender error".to_string(),
            ))),
            Err(e) => Err(PaymentError::Engine(EngineError::ValidationError(format!(
                "Failed to send command to client actor: {:?}",
                e
            )))),
        }
    }

    /// Get state for a specific client (uses global registry lookup)
    pub async fn get_state(&self, client_id: u16) -> Result<Option<AccountState>, PaymentError> {
        let actor_name = if self.namespace.is_empty() {
            format!("client-{}", client_id)
        } else {
            format!("{}-client-{}", self.namespace, client_id)
        };

        if let Some(actor_ref) = ActorRef::<ClientActorMessage>::where_is(actor_name) {
            match actor_ref
                .call(
                    ClientActorMessage::GetState,
                    Some(std::time::Duration::from_millis(100)),
                )
                .await
            {
                Ok(CallResult::Success(state)) => Ok(Some(state)),
                Ok(CallResult::Timeout) => Err(PaymentError::Engine(EngineError::ValidationError(
                    "Actor call timeout".to_string(),
                ))),
                Ok(CallResult::SenderError) => Err(PaymentError::Engine(
                    EngineError::ValidationError("Actor sender error".to_string()),
                )),
                Err(e) => Err(PaymentError::Engine(EngineError::ValidationError(format!(
                    "Failed to get state from client actor: {:?}",
                    e
                )))),
            }
        } else {
            Ok(None)
        }
    }

    /// Get all client states that we've processed (for final output)
    ///
    /// This uses our local tracking (processed_clients) to know which clients
    /// to query, then looks them up in the global registry.
    pub async fn get_all_states(
        &self,
    ) -> Result<std::collections::HashMap<u16, AccountState>, PaymentError> {
        let mut states = std::collections::HashMap::new();

        let client_ids: Vec<u16> = {
            let clients = self.processed_clients.lock().unwrap();
            clients.iter().copied().collect()
        };

        for client_id in client_ids {
            if let Ok(Some(state)) = self.get_state(client_id).await {
                states.insert(client_id, state);
            } else {
                tracing::warn!("Failed to get state for client {}", client_id);
            }
        }

        Ok(states)
    }

    /// Shutdown all client actors that we've processed
    ///
    /// Note: In a distributed system, this only stops actors we've tracked locally.
    /// Other nodes might have their own references to the same actors.
    pub async fn shutdown_all(&self) {
        let client_ids: Vec<u16> = {
            let clients = self.processed_clients.lock().unwrap();
            clients.iter().copied().collect()
        };

        for client_id in client_ids {
            let actor_name = if self.namespace.is_empty() {
                format!("client-{}", client_id)
            } else {
                format!("{}-client-{}", self.namespace, client_id)
            };
            if let Some(actor_ref) = ActorRef::<ClientActorMessage>::where_is(actor_name) {
                actor_ref.stop(None);
            }
        }

        self.processed_clients.lock().unwrap().clear();
    }
}
