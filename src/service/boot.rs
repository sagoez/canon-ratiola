use crate::adapter::{ClientRegistry, InMemoryDisputeIndex, InMemoryJournal};
use std::sync::Arc;

/// Setup the payment system and return a client registry (Akka-style)
///
/// This creates all the infrastructure:
/// - InMemoryJournal (shared event store - like Akka Persistence)
/// - InMemoryDisputeIndex (shared infrastructure index)
/// - ClientRegistry (spawns client actors on-demand)
///
/// Architecture:
/// - CSV/Kafka → Orchestrator → ClientRegistry → ClientActor (per client) → InMemoryJournal
/// - Each ClientActor writes directly to shared journal (no central actor)
/// - DisputeIndex maintained via callbacks (infrastructure concern)
/// - Simple, efficient, ready for database replacement
pub async fn boot() -> ClientRegistry {
    let journal: Arc<dyn crate::port::Journal + Send + Sync> = Arc::new(InMemoryJournal::new());
    let dispute_index: Arc<dyn crate::port::DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());

    tracing::info!("Payment system initialized");

    ClientRegistry::new(journal, dispute_index)
}
