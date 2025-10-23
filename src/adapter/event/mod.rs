mod chargebacked_handler;
mod deposited_handler;
mod disputed_handler;
mod resolved_handler;
mod withdrawn_handler;

use crate::domain::{AccountState, TransactionTypeEvent};
use crate::{domain::EventEnvelope, port::EventHandler};

impl EventHandler for EventEnvelope {
    fn apply(&self, state: &AccountState) -> Option<AccountState> {
        match &self.event {
            TransactionTypeEvent::Deposited(event) => event.apply(state),
            TransactionTypeEvent::Withdrawn(event) => event.apply(state),
            TransactionTypeEvent::Disputed(event) => event.apply(state),
            TransactionTypeEvent::Resolved(event) => event.apply(state),
            TransactionTypeEvent::Chargebacked(event) => event.apply(state),
        }
    }
}
