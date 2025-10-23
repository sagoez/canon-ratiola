use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::DeduplicationKey;

/// Metadata needed to construct an event envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub client_id: u16,
    pub tx_id: u32,
    pub deduplication_key: DeduplicationKey,
    pub timestamp: DateTime<Utc>,
}
