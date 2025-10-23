use crate::{
    domain::{
        AccountState, Deposit, Deposited, PaymentError, TransactionError, TransactionTypeEvent,
    },
    port::{CommandHandler, TransactionLookup},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
impl CommandHandler for Deposit {
    type Resource = ();
    type Entity = ();

    async fn load(
        &self,
        _stale_state: &AccountState,
        _lookup: &dyn TransactionLookup,
    ) -> Result<Self::Resource, PaymentError> {
        Ok(())
    }

    fn validate(
        &self,
        _state: &AccountState,
        _resource: &Self::Resource,
    ) -> Result<Self::Entity, PaymentError> {
        if self.amount <= 0.0 {
            return Err(PaymentError::Transaction(TransactionError::InvalidAmount));
        }

        Ok(())
    }

    fn emit(
        &self,
        _state: &AccountState,
        _entity: &Self::Entity,
        _resource: &Self::Resource,
        _timestamp: DateTime<Utc>,
    ) -> Result<Vec<TransactionTypeEvent>, PaymentError> {
        Ok(vec![TransactionTypeEvent::Deposited(Deposited {
            client_id: self.client_id,
            tx_id: self.tx_id,
            amount: self.amount,
        })])
    }

    async fn effect(
        &self,
        _previous_state: &AccountState,
        _state: &AccountState,
        _resource: &Self::Resource,
        _entity: &Self::Entity,
        _timestamp: DateTime<Utc>,
    ) -> Result<(), PaymentError> {
        Ok(())
    }
}
