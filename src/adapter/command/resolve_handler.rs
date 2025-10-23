use crate::{
    domain::{
        AccountState, EngineError, PaymentError, Resolve, Resolved, TransactionError,
        TransactionTypeEvent,
    },
    port::{CommandHandler, TransactionLookup},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
impl CommandHandler for Resolve {
    type Resource = (TransactionTypeEvent, bool);
    type Entity = f64; // Amount to resolve

    async fn load(
        &self,
        _stale_state: &AccountState,
        lookup: &dyn TransactionLookup,
    ) -> Result<Self::Resource, PaymentError> {
        // Load the original transaction
        let original_tx = lookup.find_transaction(self.tx_id).await?.ok_or_else(|| {
            PaymentError::Engine(EngineError::LoadingResourcesError(format!(
                "Transaction {} not found",
                self.tx_id
            )))
        })?;

        // Check if transaction is currently disputed (database concern via infrastructure)
        let is_disputed = lookup.is_disputed(self.tx_id).await?;

        Ok((original_tx, is_disputed))
    }

    fn validate(
        &self,
        _state: &AccountState,
        resource: &Self::Resource,
    ) -> Result<Self::Entity, PaymentError> {
        let (original_tx, is_disputed) = resource;

        // Extract amount from original transaction (must be deposit or withdrawal)
        let amount = match original_tx {
            TransactionTypeEvent::Deposited(d) => d.amount,
            TransactionTypeEvent::Withdrawn(w) => w.amount,
            _ => {
                return Err(PaymentError::Transaction(
                    TransactionError::InvalidTransactionType,
                ));
            }
        };

        // Per spec: "If the tx isn't under dispute, you can ignore the resolve"
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
        Ok(vec![TransactionTypeEvent::Resolved(Resolved {
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
