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
async fn test_withdrawal_validates_sufficient_funds() {
    let withdrawal = Withdraw {
        client_id: 1,
        tx_id: 2,
        amount: 150.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 100.0,
        held: 0.0,
        total: 100.0,
        last_activity: chrono::Utc::now(),
    });

    let lookup = create_mock_lookup();
    let resource = withdrawal.load(&state, lookup.as_ref()).await.unwrap();

    let result = withdrawal.validate(&state, &resource);
    assert!(
        result.is_err(),
        "Should reject withdrawal with insufficient funds"
    );
}

#[tokio::test]
async fn test_withdrawal_allows_exact_balance() {
    let withdrawal = Withdraw {
        client_id: 1,
        tx_id: 2,
        amount: 100.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 100.0,
        held: 0.0,
        total: 100.0,
        last_activity: chrono::Utc::now(),
    });

    let lookup = create_mock_lookup();
    let resource = withdrawal.load(&state, lookup.as_ref()).await.unwrap();
    let entity = withdrawal.validate(&state, &resource).unwrap();
    let events = withdrawal
        .emit(&state, &entity, &resource, chrono::Utc::now())
        .unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        TransactionTypeEvent::Withdrawn(w) => {
            assert_eq!(w.amount, 100.0);
        }
        _ => panic!("Expected Withdrawn event"),
    }
}

#[tokio::test]
async fn test_withdrawal_rejects_negative_amount() {
    let withdrawal = Withdraw {
        client_id: 1,
        tx_id: 2,
        amount: -50.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 100.0,
        held: 0.0,
        total: 100.0,
        last_activity: chrono::Utc::now(),
    });

    let lookup = create_mock_lookup();
    let resource = withdrawal.load(&state, lookup.as_ref()).await.unwrap();

    let result = withdrawal.validate(&state, &resource);
    assert!(
        result.is_err(),
        "Should reject negative withdrawal amounts"
    );
}

#[tokio::test]
async fn test_withdrawal_rejects_when_frozen() {
    let withdrawal = Withdraw {
        client_id: 1,
        tx_id: 2,
        amount: 50.0,
    };

    let state = AccountState::Frozen(FrozenAccountState {
        available: 100.0,
        held: 0.0,
        total: 100.0,
        last_activity: chrono::Utc::now(),
    });

    let lookup = create_mock_lookup();
    let resource = withdrawal.load(&state, lookup.as_ref()).await.unwrap();

    let result = withdrawal.validate(&state, &resource);
    assert!(result.is_err(), "Should reject withdrawal on frozen account");
}

