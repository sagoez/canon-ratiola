use crate::{
    domain::{DeduplicationKey, EventEnvelope, EventMetadata, PaymentError, TransactionTypeEvent},
    port::Journal,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct JournalData {
    events: Vec<Arc<EventEnvelope>>,
    deduplication_index: HashMap<DeduplicationKey, Arc<EventEnvelope>>,
    tx_id_index: HashMap<u32, Vec<Arc<EventEnvelope>>>,
    sequence_counter: u64,
}

/// In-memory journal implementation
pub struct InMemoryJournal {
    data: Arc<RwLock<JournalData>>,
}

impl InMemoryJournal {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(JournalData {
                events: Vec::new(),
                deduplication_index: HashMap::new(),
                tx_id_index: HashMap::new(),
                sequence_counter: 0,
            })),
        }
    }
}

#[async_trait]
impl Journal for InMemoryJournal {
    async fn append(
        &self,
        event: TransactionTypeEvent,
        metadata: EventMetadata,
    ) -> Result<EventEnvelope, PaymentError> {
        let deduplication_key = metadata.deduplication_key;

        let mut data = self.data.write().await;

        if let Some(existing) = data.deduplication_index.get(&deduplication_key) {
            return Ok((**existing).clone());
        }

        data.sequence_counter += 1;
        let sequence_nr = data.sequence_counter;

        let envelope = Arc::new(EventEnvelope {
            sequence_nr,
            event,
            timestamp: metadata.timestamp,
            client_id: metadata.client_id,
            tx_id: metadata.tx_id,
            deduplication_key: deduplication_key.clone(),
        });

        data.events.push(envelope.clone());
        data.deduplication_index
            .insert(deduplication_key, envelope.clone());
        data.tx_id_index
            .entry(metadata.tx_id)
            .or_insert_with(Vec::new)
            .push(envelope.clone());

        Ok((*envelope).clone())
    }

    async fn replay(&self, from_sequence: Option<u64>) -> Result<Vec<EventEnvelope>, PaymentError> {
        let data = self.data.read().await;
        let from = from_sequence.unwrap_or(0);

        Ok(data
            .events
            .iter()
            .filter(|e| e.sequence_nr >= from)
            .map(|arc| (**arc).clone())
            .collect())
    }

    async fn highest_sequence(&self) -> Result<Option<u64>, PaymentError> {
        let data = self.data.read().await;
        if data.sequence_counter == 0 {
            Ok(None)
        } else {
            Ok(Some(data.sequence_counter))
        }
    }

    async fn find_by_tx_id(&self, tx_id: u32) -> Result<Vec<EventEnvelope>, PaymentError> {
        let data = self.data.read().await;
        Ok(data
            .tx_id_index
            .get(&tx_id)
            .map(|arcs| arcs.iter().map(|arc| (**arc).clone()).collect())
            .unwrap_or_default())
    }
}

impl Default for InMemoryJournal {
    fn default() -> Self {
        Self::new()
    }
}
