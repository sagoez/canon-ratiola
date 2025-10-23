use payment::domain::*;
use payment::port::EventHandler;

#[test]
fn test_chargebacked_reverses_and_freezes() {
    let event = Chargebacked {
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
        AccountState::Frozen(frozen) => {
            assert_eq!(frozen.available, 0.0);
            assert_eq!(frozen.held, 0.0);
            assert_eq!(frozen.total, 0.0);
        }
        _ => panic!("Expected Frozen state"),
    }
}

#[test]
fn test_chargebacked_with_insufficient_held_funds() {
    let event = Chargebacked {
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
        "Should reject chargeback with insufficient held funds"
    );
}

#[test]
fn test_chargebacked_on_already_frozen_account() {
    let event = Chargebacked {
        client_id: 1,
        tx_id: 1,
        amount: 50.0,
    };

    let state = AccountState::Frozen(FrozenAccountState {
        available: 100.0,
        held: 50.0,
        total: 150.0,
        last_activity: chrono::Utc::now(),
    });

    let new_state = event.apply(&state).expect("Should apply successfully");

    match new_state {
        AccountState::Frozen(frozen) => {
            assert_eq!(frozen.available, 100.0);
            assert_eq!(frozen.held, 0.0);
            assert_eq!(frozen.total, 100.0);
        }
        _ => panic!("Expected Frozen state"),
    }
}

#[test]
fn test_chargebacked_partial_amount() {
    let event = Chargebacked {
        client_id: 1,
        tx_id: 1,
        amount: 50.0,
    };

    let state = AccountState::Active(ActiveAccountState {
        available: 100.0,
        held: 50.0,
        total: 150.0,
        last_activity: chrono::Utc::now(),
    });

    let new_state = event.apply(&state).expect("Should apply successfully");

    match new_state {
        AccountState::Frozen(frozen) => {
            assert_eq!(frozen.available, 100.0);
            assert_eq!(frozen.held, 0.0);
            assert_eq!(frozen.total, 100.0);
        }
        _ => panic!("Expected Frozen state"),
    }
}

