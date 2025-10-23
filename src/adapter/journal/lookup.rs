use crate::domain::{PaymentError, TransactionTypeEvent};
use crate::port::{DisputeIndex, Journal, TransactionLookup};
use async_trait::async_trait;
use std::sync::Arc;

/// Transaction lookup implementation using Journal and DisputeIndex
pub struct JournalTransactionLookup {
    journal: Arc<dyn Journal + Send + Sync>,
    dispute_index: Arc<dyn DisputeIndex>,
}

impl JournalTransactionLookup {
    pub fn new(
        journal: Arc<dyn Journal + Send + Sync>,
        dispute_index: Arc<dyn DisputeIndex>,
    ) -> Self {
        Self {
            journal,
            dispute_index,
        }
    }
}

#[async_trait]
impl TransactionLookup for JournalTransactionLookup {
    async fn find_transaction(
        &self,
        tx_id: u32,
    ) -> Result<Option<TransactionTypeEvent>, PaymentError> {
        let events = self.journal.find_by_tx_id(tx_id).await?;

        for envelope in events {
            match &envelope.event {
                TransactionTypeEvent::Deposited(_) | TransactionTypeEvent::Withdrawn(_) => {
                    return Ok(Some(envelope.event.clone()));
                }
                _ => continue,
            }
        }

        Ok(None)
    }

    async fn is_disputed(&self, tx_id: u32) -> Result<bool, PaymentError> {
        // Query the separate DisputeIndex (O(1) lookup)
        self.dispute_index.is_disputed(tx_id).await
    }
}
