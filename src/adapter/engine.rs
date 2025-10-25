use crate::{
    domain::{
        AccountState, CommandMetadata, EngineError, EventEnvelope, EventMetadata, PaymentError,
        TransactionTypeCommand,
    },
    port::{Engine, EventCallback, EventHandler, Journal, Processor},
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

/// Context for the Engine containing current state and journal
pub struct EngineContext {
    /// The journal for persisting events
    pub journal: Arc<dyn Journal + Send + Sync>,
    /// Current state of the account
    pub current_state: AccountState,
}

/// The main payment engine implementation
pub struct PaymentEngine {
    processor: Arc<dyn Processor>,
    /// User-provided callbacks (optional, for custom business logic)
    user_callbacks: Vec<Arc<dyn EventCallback>>,
}

impl PaymentEngine {
    pub fn new(processor: Arc<dyn Processor>) -> Self {
        Self {
            processor,
            user_callbacks: Vec::with_capacity(10),
        }
    }

    /// Add a user callback to be invoked after event persistence
    ///
    /// These callbacks are invoked AFTER infrastructure callbacks (maintained by Journal).
    /// Use this for custom business logic like notifications, analytics, etc.
    pub fn with_callback(mut self, callback: Arc<dyn EventCallback>) -> Self {
        self.user_callbacks.push(callback);
        self
    }

    /// Invoke callbacks after event persistence
    ///
    /// Two types of callbacks are invoked:
    /// 1. Infrastructure callbacks (mandatory) - handled by Journal (e.g., dispute index)
    /// 2. User callbacks (optional) - custom business logic provided by user
    async fn invoke_callbacks(
        &self,
        envelope: &EventEnvelope,
        context: &EngineContext,
    ) -> Result<(), PaymentError> {
        use crate::domain::TransactionTypeEvent;
        use crate::port::CallbackContext;

        // Infrastructure callbacks are already handled by Journal.append()
        // (e.g., maintaining dispute index in InMemoryJournal)

        // Create callback context for user callbacks
        let callback_ctx = CallbackContext {
            journal: context.journal.clone(),
            envelope: envelope.clone(),
        };

        // Invoke user-provided callbacks
        for callback in &self.user_callbacks {
            // Dispatch to appropriate callback method based on event type
            match &envelope.event {
                TransactionTypeEvent::Deposited(event) => {
                    callback.on_deposited(event, &callback_ctx).await?;
                }
                TransactionTypeEvent::Withdrawn(event) => {
                    callback.on_withdrawn(event, &callback_ctx).await?;
                }
                TransactionTypeEvent::Disputed(event) => {
                    callback.on_disputed(event, &callback_ctx).await?;
                }
                TransactionTypeEvent::Resolved(event) => {
                    callback.on_resolved(event, &callback_ctx).await?;
                }
                TransactionTypeEvent::Chargebacked(event) => {
                    callback.on_chargebacked(event, &callback_ctx).await?;
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Engine for PaymentEngine {
    type Context = EngineContext;

    /// Process a command by orchestrating the following steps:
    /// 1. Async load phase (can query external state, use snapshot)
    /// 2. Validation phase (apply business rules to current state)
    /// 3. Persist event to journal (journal assigns sequence number atomically)
    /// 4. Apply event to state (functional - returns new state)
    /// 5. Execute effects (with new state)
    ///
    /// INFRASTRUCTURE CONTRACT (caller's responsibility):
    /// - Caller MUST provide serialization (e.g., actor model with sequential processing)
    /// - Caller MUST verify sequence number ordering after persistence
    /// - Caller MUST update state atomically after successful processing
    ///
    /// This separation keeps the engine pure (stateless business logic) while
    /// pushing ordering guarantees to infrastructure (ClientActor).
    ///
    /// Returns (EventEnvelope, NewState) - includes sequence number for verification
    async fn process_command(
        &self,
        command: TransactionTypeCommand,
        metadata: CommandMetadata,
        context: &Self::Context,
    ) -> Result<(EventEnvelope, AccountState), PaymentError> {
        // 1. Load phase: query dependencies (e.g., lookup disputed transaction)
        //    Uses snapshot of current state - this can be slow (I/O)
        //    Caller's serialization ensures state doesn't change during this
        let stale_state = context.current_state.clone();
        let validate_fn = self.processor.load(command.clone(), &stale_state).await?;

        // 2. Validation phase: apply business rules to CURRENT state
        //    Infrastructure guarantee: state hasn't changed since load phase
        //    (ClientActor's sequential processing ensures this)
        let directive = validate_fn.apply(&context.current_state)?;

        // 3. Persistence phase: append event to journal
        //    Journal handles:
        //    - Idempotency check via deduplication_key
        //    - Atomic sequence number assignment (under journal's write lock)
        //    - Returns existing envelope if duplicate
        let event = directive
            .events
            .into_iter()
            .next()
            .ok_or(PaymentError::Engine(EngineError::NoEvents))?;

        let event_metadata = EventMetadata {
            client_id: command.client_id(),
            tx_id: command.tx_id(),
            deduplication_key: metadata.deduplication_key,
            timestamp: Utc::now(),
        };

        let envelope = context.journal.append(event, event_metadata).await?;

        // 3.5. Infrastructure callbacks: notify about event persistence
        //      This is where infrastructure concerns (like dispute index) are updated
        self.invoke_callbacks(&envelope, context).await?;

        // 4. State transition: apply event to get new state
        //    This is functional (pure) - returns new state, doesn't mutate
        let new_state = envelope
            .apply(&context.current_state)
            .ok_or(PaymentError::Engine(EngineError::StateTransitionFailed))?;

        // 5. Effects: execute side effects with new state
        for effect in directive.effects {
            effect.execute(&new_state).await?;
        }

        Ok((envelope, new_state))
    }

    fn processor(&self) -> &dyn Processor {
        self.processor.as_ref()
    }
}
