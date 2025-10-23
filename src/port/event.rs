use crate::domain::{AccountState, PaymentError, TransactionTypeEvent};
use async_trait::async_trait;

/// EventHandler is responsible for applying the event to the state.
/// It is used to apply the event to the state to reconstruct the state of the account.
///
/// EventHandler#apply is a pure function, can't be async because it should NEVER be
/// side-effectful (even if its not async, it should not have any side-effects).
pub trait EventHandler: Send {
    /// Apply the event to the state. This will run after the event is emitted and persisted.
    fn apply(&self, state: &AccountState) -> Option<AccountState>;
}

#[async_trait]
pub trait EventProcessor {
    async fn process(
        &self,
        event: TransactionTypeEvent,
        previous_state: &AccountState,
    ) -> Result<AccountState, PaymentError>;
}
