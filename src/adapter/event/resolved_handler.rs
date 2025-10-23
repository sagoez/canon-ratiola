use crate::{
    domain::{AccountState, ActiveAccountState, FrozenAccountState, Resolved},
    port::EventHandler,
};

impl EventHandler for Resolved {
    fn apply(&self, state: &AccountState) -> Option<AccountState> {
        match state {
            AccountState::Active(active) => {
                if active.held < self.amount {
                    return None;
                }
                Some(AccountState::Active(ActiveAccountState {
                    available: active.available + self.amount,
                    held: active.held - self.amount,
                    total: active.total,
                    last_activity: chrono::Utc::now(),
                }))
            }
            AccountState::Frozen(frozen) => {
                if frozen.held < self.amount {
                    return None;
                }
                Some(AccountState::Frozen(FrozenAccountState {
                    available: frozen.available + self.amount,
                    held: frozen.held - self.amount,
                    total: frozen.total,
                    last_activity: chrono::Utc::now(),
                }))
            }
        }
    }
}
