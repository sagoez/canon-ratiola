use crate::{
    domain::{AccountState, ActiveAccountState, Disputed, FrozenAccountState},
    port::EventHandler,
};

impl EventHandler for Disputed {
    fn apply(&self, state: &AccountState) -> Option<AccountState> {
        match state {
            AccountState::Active(active) => Some(AccountState::Active(ActiveAccountState {
                available: active.available - self.amount,
                held: active.held + self.amount,
                total: active.total,
                last_activity: chrono::Utc::now(),
            })),
            AccountState::Frozen(frozen) => Some(AccountState::Frozen(FrozenAccountState {
                available: frozen.available - self.amount,
                held: frozen.held + self.amount,
                total: frozen.total,
                last_activity: chrono::Utc::now(),
            })),
        }
    }
}
