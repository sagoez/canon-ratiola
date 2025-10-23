use crate::{
    domain::{
        AccountState, Chargeback, Chargebacked, EngineError, PaymentError, TransactionError,
        TransactionTypeEvent,
    },
    port::{CommandHandler, TransactionLookup},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
impl CommandHandler for Chargeback {
    type Resource = (TransactionTypeEvent, bool);
    type Entity = f64;

    async fn load(
        &self,
        _stale_state: &AccountState,
        lookup: &dyn TransactionLookup,
    ) -> Result<Self::Resource, PaymentError> {
        let original_tx = lookup.find_transaction(self.tx_id).await?.ok_or_else(|| {
            PaymentError::Engine(EngineError::LoadingResourcesError(format!(
                "Transaction {} not found",
                self.tx_id
            )))
        })?;

        let is_disputed = lookup.is_disputed(self.tx_id).await?;

        Ok((original_tx, is_disputed))
    }

    fn validate(
        &self,
        _state: &AccountState,
        resource: &Self::Resource,
    ) -> Result<Self::Entity, PaymentError> {
        let (original_tx, is_disputed) = resource;

        let amount = match original_tx {
            TransactionTypeEvent::Deposited(d) => d.amount,
            TransactionTypeEvent::Withdrawn(w) => w.amount,
            _ => {
                return Err(PaymentError::Transaction(
                    TransactionError::InvalidTransactionType,
                ));
            }
        };

        // Per spec: "If the tx isn't under dispute, you can ignore the chargeback"
        if !is_disputed {
            return Err(PaymentError::Transaction(
                TransactionError::InvalidTransactionType,
            ));
        }

        Ok(amount)
    }

    fn emit(
        &self,
        _state: &AccountState,
        entity: &Self::Entity,
        _resource: &Self::Resource,
        _timestamp: DateTime<Utc>,
    ) -> Result<Vec<TransactionTypeEvent>, PaymentError> {
        Ok(vec![TransactionTypeEvent::Chargebacked(Chargebacked {
            client_id: self.client_id,
            tx_id: self.tx_id,
            amount: *entity,
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
