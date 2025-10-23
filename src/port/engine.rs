use crate::domain::{
    AccountState, CommandMetadata, Directive, EventEnvelope, PaymentError, TransactionTypeCommand,
};
use async_trait::async_trait;

/// Engine orchestrates command processing with exclusive state access
///
/// Responsibilities:
/// - Global ordering via sequence numbers
/// - Deduplication via CommandMetadata (idempotent at-least-once)
/// - Event persistence via Journal
/// - Exclusive access to current state (single-threaded like Akka actor)
#[async_trait]
pub trait Engine {
    type Context;

    /// Process a command with ordering and delivery guarantees
    ///
    /// The engine orchestrates:
    /// 1. processor.load(cmd, stale_state) -> returns Validate function
    /// 2. validate_fn(actual_state) -> returns Directive (events + effects)
    /// 3. Persist events to journal (handles idempotency & sequence assignment)
    /// 4. Apply events to state (functional)
    /// 5. Execute effects
    ///
    /// Returns (EventEnvelope, NewState) - caller is responsible for updating state
    async fn process_command(
        &self,
        command: TransactionTypeCommand,
        metadata: CommandMetadata,
        context: &Self::Context,
    ) -> Result<(EventEnvelope, AccountState), PaymentError>;

    /// Get the command processor/loader
    fn processor(&self) -> &dyn Processor;
}

/// Processor dispatches commands to handlers
#[async_trait]
pub trait Processor: Send + Sync {
    /// Load command with stale state
    ///
    /// This can be slow (DB queries, HTTP calls, etc.) and uses potentially stale state.
    /// Returns a ValidateFn that will be called later with actual state.
    async fn load(
        &self,
        command: TransactionTypeCommand,
        stale_state: &AccountState,
    ) -> Result<Box<dyn ValidateFn>, PaymentError>;
}

/// The Validate function returned by Processor::load
///
/// This function is called with exclusive access to actual state and must be FAST.
pub trait ValidateFn: Send {
    /// Validate against actual state and return directive
    ///
    /// This must be FAST - no async, no I/O, just business logic.
    /// Takes actual state, returns events and effects.
    /// Sequence numbers are assigned by the Journal during persistence.
    fn apply(&self, actual_state: &AccountState) -> Result<Directive, PaymentError>;
}

/// An effect to execute after event persistence
#[async_trait]
pub trait EffectFn: Send + Sync {
    async fn execute(&self, new_state: &AccountState) -> Result<(), PaymentError>;
}
