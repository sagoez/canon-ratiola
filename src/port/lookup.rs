use crate::domain::{PaymentError, TransactionTypeEvent};
use async_trait::async_trait;

/// TransactionLookup provides read-only access to historical transactions
///
/// This is used during the command "load" phase to look up original transactions
/// for disputes, resolves, and chargebacks.
#[async_trait]
pub trait TransactionLookup: Send + Sync {
    /// Find the original transaction by tx_id
    ///
    /// Returns the first Deposited or Withdrawn event for this tx_id.
    /// Returns None if transaction doesn't exist.
    async fn find_transaction(
        &self,
        tx_id: u32,
    ) -> Result<Option<TransactionTypeEvent>, PaymentError>;

    /// Check if a transaction is currently under dispute
    ///
    /// Returns true if the transaction has been disputed and not yet resolved/chargebacked.
    /// This is a database-level concern, not a state concern.
    async fn is_disputed(&self, tx_id: u32) -> Result<bool, PaymentError>;
}
