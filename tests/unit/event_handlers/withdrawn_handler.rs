use payment::domain::*;
use payment::port::EventHandler;

#[test]
fn test_withdrawn_decreases_balance() {
    let event = Withdrawn {
        client_id: 1,
        tx_id: 2,
        amount: 30.0,
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
            assert_eq!(active.available, 70.0);
            assert_eq!(active.total, 70.0);
        }
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_withdrawn_on_frozen_account_rejected() {
    let event = Withdrawn {
        client_id: 1,
        tx_id: 2,
        amount: 30.0,
    };

    let state = AccountState::Frozen(FrozenAccountState {
        available: 100.0,
        held: 0.0,
        total: 100.0,
        last_activity: chrono::Utc::now(),
    });

    let result = event.apply(&state);
    assert!(
        result.is_none(),
        "Withdrawal should be rejected on frozen account"
    );
}

#[test]
fn test_withdrawn_exact_balance() {
    let event = Withdrawn {
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

    let new_state = event.apply(&state).expect("Should apply successfully");

    match new_state {
        AccountState::Active(active) => {
            assert_eq!(active.available, 0.0);
            assert_eq!(active.total, 0.0);
        }
        _ => panic!("Expected Active state"),
    }
}

