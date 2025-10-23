use payment::domain::*;
use payment::port::EventHandler;

#[test]
fn test_deposited_updates_balance() {
    let event = Deposited {
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

    let new_state = event.apply(&state).expect("Should apply successfully");

    match new_state {
        AccountState::Active(active) => {
            assert_eq!(active.available, 100.0);
            assert_eq!(active.held, 0.0);
            assert_eq!(active.total, 100.0);
        }
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_deposited_on_frozen_account() {
    let event = Deposited {
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
        .expect("Frozen accounts can receive deposits");

    match new_state {
        AccountState::Frozen(frozen) => {
            assert_eq!(frozen.available, 150.0);
            assert_eq!(frozen.total, 150.0);
        }
        _ => panic!("Should remain frozen"),
    }
}

#[test]
fn test_deposited_accumulates_correctly() {
    let event = Deposited {
        client_id: 1,
        tx_id: 2,
        amount: 75.5,
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
            assert_eq!(active.available, 175.5);
            assert_eq!(active.total, 175.5);
        }
        _ => panic!("Expected Active state"),
    }
}

