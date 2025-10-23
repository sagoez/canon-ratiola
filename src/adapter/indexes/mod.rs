use crate::domain::*;
use crate::port::DisputeIndex;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory implementation of DisputeIndex using HashMap
///
/// For production, use a database-backed implementation (e.g., PostgresDisputeIndex)
pub struct InMemoryDisputeIndex {
    disputed_txs: Arc<RwLock<HashMap<u32, f64>>>, // tx_id -> amount
}

impl InMemoryDisputeIndex {
    pub fn new() -> Self {
        Self {
            disputed_txs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryDisputeIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DisputeIndex for InMemoryDisputeIndex {
    async fn is_disputed(&self, tx_id: u32) -> Result<bool, PaymentError> {
        let disputed = self.disputed_txs.read().await;
        Ok(disputed.contains_key(&tx_id))
    }

    async fn mark_disputed(&self, tx_id: u32, amount: f64) -> Result<(), PaymentError> {
        let mut disputed = self.disputed_txs.write().await;
        disputed.insert(tx_id, amount);
        Ok(())
    }

    async fn unmark_disputed(&self, tx_id: u32) -> Result<(), PaymentError> {
        let mut disputed = self.disputed_txs.write().await;
        disputed.remove(&tx_id);
        Ok(())
    }
}
