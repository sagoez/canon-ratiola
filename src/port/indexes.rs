use crate::domain::PaymentError;
use async_trait::async_trait;

/// Port for querying dispute status
///
/// This is a separate infrastructure concern from the Journal.
/// Implementations can use in-memory HashMap, Redis, PostgreSQL, etc.
#[async_trait]
pub trait DisputeIndex: Send + Sync {
    /// Check if a transaction is currently disputed (should be O(1) or close)
    async fn is_disputed(&self, tx_id: u32) -> Result<bool, PaymentError>;

    /// Mark a transaction as disputed (called by infrastructure callbacks)
    async fn mark_disputed(&self, tx_id: u32, amount: f64) -> Result<(), PaymentError>;

    /// Unmark a transaction as disputed (called by infrastructure callbacks)
    async fn unmark_disputed(&self, tx_id: u32) -> Result<(), PaymentError>;
}
