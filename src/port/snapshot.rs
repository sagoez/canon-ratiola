use crate::domain::{AccountState, PaymentError};
use async_trait::async_trait;

/// Snapshotter is responsible for saving and loading the state at a given point in time
/// of the account.
#[async_trait]
pub trait Snapshotter {
    /// Save the current state to the snapshot
    async fn save(&self, sequence: u64, state: AccountState) -> Result<(), PaymentError>;

    /// Load the state from the snapshot
    async fn load(&self) -> Result<Option<(u64, AccountState)>, PaymentError>;
}
