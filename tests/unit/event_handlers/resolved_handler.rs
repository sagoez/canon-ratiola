use payment::domain::*;
use payment::port::EventHandler;

#[test]
fn test_resolved_releases_held_funds() {
    let event = Resolved {
        client_id: 1,
        tx_id: 1,
        amount: 100.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 0.0,
        held: 100.0,
        total: 100.0,
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
fn test_resolved_with_insufficient_held_funds() {
    let event = Resolved {
        client_id: 1,
        tx_id: 1,
        amount: 150.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 0.0,
        held: 100.0,
        total: 100.0,
        last_activity: chrono::Utc::now(),
    });

    let result = event.apply(&state);
    assert!(
        result.is_none(),
        "Should reject resolve with insufficient held funds"
    );
}

#[test]
fn test_resolved_partial_amount() {
    let event = Resolved {
        client_id: 1,
        tx_id: 1,
        amount: 50.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 0.0,
        held: 100.0,
        total: 100.0,
        last_activity: chrono::Utc::now(),
    });

    let new_state = event.apply(&state).expect("Should apply successfully");

    match new_state {
        AccountState::Active(active) => {
            assert_eq!(active.available, 50.0);
            assert_eq!(active.held, 50.0);
            assert_eq!(active.total, 100.0);
        }
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_resolved_on_frozen_account() {
    let event = Resolved {
        client_id: 1,
        tx_id: 1,
        amount: 50.0,
    };

    let state = AccountState::Frozen(FrozenAccountState {
        available: 0.0,
        held: 100.0,
        total: 100.0,
        last_activity: chrono::Utc::now(),
    });

    let new_state = event.apply(&state).expect("Should apply successfully");

    match new_state {
        AccountState::Frozen(frozen) => {
            assert_eq!(frozen.available, 50.0);
            assert_eq!(frozen.held, 50.0);
            assert_eq!(frozen.total, 100.0);
        }
        _ => panic!("Should remain frozen"),
    }
}

