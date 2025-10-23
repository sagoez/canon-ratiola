use crate::{domain::TransactionTypeEvent, port::EffectFn};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeduplicationKey(String);

impl DeduplicationKey {
    pub fn new(identifier: String) -> Self {
        Self(identifier)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Envelope wrapping an event with ordering metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Global sequence number for ordering guarantees
    pub sequence_nr: u64,
    /// The domain event
    pub event: TransactionTypeEvent,
    /// When the event was processed
    pub timestamp: DateTime<Utc>,
    /// Client the event belongs to
    pub client_id: u16,
    /// Transaction ID from the domain
    pub tx_id: u32,
    /// Deduplication key from the command source (Kafka offset, API request ID, etc.)
    pub deduplication_key: DeduplicationKey,
}

/// Metadata about the command for deduplication
///
/// This allows the engine to work with any message source:
/// - Kafka: use partition + offset
/// - HTTP API: use idempotency key header
/// - File/CSV: use line number
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandMetadata {
    /// Opaque identifier that uniquely identifies this command instance
    /// Examples: "kafka:0:1234", "request:abc-123", "file:line:42, csv:row:123"
    pub deduplication_key: DeduplicationKey,
}

/// Directive contains events to persist and effects to execute
pub struct Directive {
    /// Events to persist to journal (without sequence numbers yet)
    pub events: Vec<TransactionTypeEvent>,
    /// Effects to execute after persistence (async, can be slow)
    pub effects: Vec<Box<dyn EffectFn>>,
}
