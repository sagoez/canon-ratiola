use payment::adapter::{InMemoryDisputeIndex, InMemoryJournal, JournalTransactionLookup};
use payment::domain::*;
use payment::port::{CommandHandler, DisputeIndex};
use std::sync::Arc;

fn create_mock_lookup() -> Arc<JournalTransactionLookup> {
    let journal: Arc<dyn payment::port::Journal + Send + Sync> = Arc::new(InMemoryJournal::new());
    let dispute_index: Arc<dyn DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());
    Arc::new(JournalTransactionLookup::new(journal, dispute_index))
}

#[tokio::test]
async fn test_dispute_requires_existing_transaction() {
    let dispute = Dispute {
        client_id: 1,
        tx_id: 999, // Non-existent transaction
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 100.0,
        held: 0.0,
        total: 100.0,
        last_activity: chrono::Utc::now(),
    });

    let lookup = create_mock_lookup();

    // Load phase should fail - transaction doesn't exist
    let result = dispute.load(&state, lookup.as_ref()).await;
    assert!(
        result.is_err(),
        "Should fail to load non-existent transaction"
    );
}

