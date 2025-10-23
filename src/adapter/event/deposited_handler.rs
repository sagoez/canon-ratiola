use crate::{
    domain::{AccountState, ActiveAccountState, Deposited, FrozenAccountState},
    port::EventHandler,
};

impl EventHandler for Deposited {
    fn apply(&self, state: &AccountState) -> Option<AccountState> {
        match state {
            AccountState::Active(active) => Some(AccountState::Active(ActiveAccountState {
                available: active.available + self.amount,
                held: active.held,
                total: active.total + self.amount,
                last_activity: chrono::Utc::now(),
            })),
            AccountState::Frozen(frozen) => Some(AccountState::Frozen(FrozenAccountState {
                available: frozen.available + self.amount,
                held: frozen.held,
                total: frozen.total + self.amount,
                last_activity: chrono::Utc::now(),
            })),
        }
    }
}
