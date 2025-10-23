use crate::{
    domain::{AccountState, ActiveAccountState, Withdrawn},
    port::EventHandler,
};

impl EventHandler for Withdrawn {
    fn apply(&self, state: &AccountState) -> Option<AccountState> {
        match state {
            AccountState::Active(active) => Some(AccountState::Active(ActiveAccountState {
                available: active.available - self.amount,
                held: active.held,
                total: active.total - self.amount,
                last_activity: chrono::Utc::now(),
            })),
            AccountState::Frozen(_) => None,
        }
    }
}
