use payment::domain::*;
use payment::port::EventHandler;

#[test]
fn test_disputed_moves_funds_to_held() {
    let event = Disputed {
        client_id: 1,
        tx_id: 1,
        amount: 100.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 100.0,
        held: 0.0,
        total: 100.0,
        last_activity: chrono::Utc::now(),
    });

    let new_state = event.apply(&state).expect("Should apply successfully");

    match new_state {
        AccountState::Active(active) => {
            assert_eq!(active.available, 0.0);
            assert_eq!(active.held, 100.0);
            assert_eq!(active.total, 100.0);
        }
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_disputed_on_frozen_account() {
    let event = Disputed {
        client_id: 1,
        tx_id: 1,
        amount: 50.0,
    };

    let state = AccountState::Frozen(FrozenAccountState {
        available: 100.0,
        held: 0.0,
        total: 100.0,
        last_activity: chrono::Utc::now(),
    });

    let new_state = event
        .apply(&state)
        .expect("Frozen accounts can initiate disputes");

    match new_state {
        AccountState::Frozen(frozen) => {
            assert_eq!(frozen.available, 50.0);
            assert_eq!(frozen.held, 50.0);
            assert_eq!(frozen.total, 100.0);
        }
        _ => panic!("Should remain frozen"),
    }
}

#[test]
fn test_disputed_partial_amount() {
    let event = Disputed {
        client_id: 1,
        tx_id: 1,
        amount: 75.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 200.0,
        held: 0.0,
        total: 200.0,
        last_activity: chrono::Utc::now(),
    });

    let new_state = event.apply(&state).expect("Should apply successfully");

    match new_state {
        AccountState::Active(active) => {
            assert_eq!(active.available, 125.0);
            assert_eq!(active.held, 75.0);
            assert_eq!(active.total, 200.0);
        }
        _ => panic!("Expected Active state"),
    }
}

