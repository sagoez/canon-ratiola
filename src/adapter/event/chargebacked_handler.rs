use crate::{
    domain::{AccountState, Chargebacked, FrozenAccountState},
    port::EventHandler,
};

impl EventHandler for Chargebacked {
    fn apply(&self, state: &AccountState) -> Option<AccountState> {
        // Defense in depth: validate even in event handler (protects replay)
        match state {
            AccountState::Active(active) => {
                // Can only chargeback if sufficient funds are held
                if active.held < self.amount {
                    return None; // Invalid: not enough held funds
                }
                Some(AccountState::Frozen(FrozenAccountState {
                    available: active.available,
                    held: active.held - self.amount,
                    total: active.total - self.amount,
                    last_activity: chrono::Utc::now(),
                }))
            }
            AccountState::Frozen(frozen) => {
                // Can only chargeback if sufficient funds are held
                if frozen.held < self.amount {
                    return None; // Invalid: not enough held funds
                }
                Some(AccountState::Frozen(FrozenAccountState {
                    available: frozen.available,
                    held: frozen.held - self.amount,
                    total: frozen.total - self.amount,
                    last_activity: chrono::Utc::now(),
                }))
            }
        }
    }
}
