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
async fn test_deposit_validation_succeeds() {
    let deposit = Deposit {
        client_id: 1,
        tx_id: 1,
        amount: 100.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 0.0,
        held: 0.0,
        total: 0.0,
        last_activity: chrono::Utc::now(),
    });

    let lookup = create_mock_lookup();
    let resource = deposit.load(&state, lookup.as_ref()).await.unwrap();
    let entity = deposit.validate(&state, &resource).unwrap();
    let events = deposit
        .emit(&state, &entity, &resource, chrono::Utc::now())
        .unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        TransactionTypeEvent::Deposited(d) => {
            assert_eq!(d.client_id, 1);
            assert_eq!(d.tx_id, 1);
            assert_eq!(d.amount, 100.0);
        }
        _ => panic!("Expected Deposited event"),
    }
}

#[tokio::test]
async fn test_deposit_rejects_negative_amount() {
    let deposit = Deposit {
        client_id: 1,
        tx_id: 1,
        amount: -50.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 0.0,
        held: 0.0,
        total: 0.0,
        last_activity: chrono::Utc::now(),
    });

    let lookup = create_mock_lookup();
    let resource = deposit.load(&state, lookup.as_ref()).await.unwrap();

    let result = deposit.validate(&state, &resource);
    assert!(result.is_err(), "Should reject negative deposit amounts");
}

#[tokio::test]
async fn test_deposit_rejects_zero_amount() {
    let deposit = Deposit {
        client_id: 1,
        tx_id: 1,
        amount: 0.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 0.0,
        held: 0.0,
        total: 0.0,
        last_activity: chrono::Utc::now(),
    });

    let lookup = create_mock_lookup();
    let resource = deposit.load(&state, lookup.as_ref()).await.unwrap();

    let result = deposit.validate(&state, &resource);
    assert!(result.is_err(), "Should reject zero deposit amounts");
}
