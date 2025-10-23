use crate::domain::{EventEnvelope, EventMetadata, PaymentError, TransactionTypeEvent};
use async_trait::async_trait;

/// Journal is responsible for appending and replaying events to the log.
/// It is used to store the events in a persistent storage and to replay them later to reconstruct the state of the account.
#[async_trait]
pub trait Journal {
    /// Append an event to the log
    ///
    /// The journal constructs the EventEnvelope by:
    /// - Assigning the next sequence number atomically
    /// - Adding the provided metadata
    /// - Wrapping the event
    ///
    /// Returns the complete EventEnvelope with assigned sequence number.
    /// Idempotent via deduplication_key - returns existing envelope if duplicate.
    async fn append(
        &self,
        event: TransactionTypeEvent,
        metadata: EventMetadata,
    ) -> Result<EventEnvelope, PaymentError>;

    /// Replay events starting from a sequence number
    /// Returns events in order
    async fn replay(&self, from_sequence: Option<u64>) -> Result<Vec<EventEnvelope>, PaymentError>;

    /// Get the highest sequence number (current position in the log)
    async fn highest_sequence(&self) -> Result<Option<u64>, PaymentError>;

    /// Find events for a specific transaction ID
    async fn find_by_tx_id(&self, tx_id: u32) -> Result<Vec<EventEnvelope>, PaymentError>;
}
