use crate::{
    domain::{
        AccountState, Dispute, Disputed, EngineError, PaymentError, TransactionError,
        TransactionTypeEvent,
    },
    port::{CommandHandler, TransactionLookup},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
impl CommandHandler for Dispute {
    type Resource = TransactionTypeEvent;
    type Entity = ();

    async fn load(
        &self,
        _stale_state: &AccountState,
        lookup: &dyn TransactionLookup,
    ) -> Result<Self::Resource, PaymentError> {
        lookup.find_transaction(self.tx_id).await?.ok_or_else(|| {
            PaymentError::Engine(EngineError::LoadingResourcesError(format!(
                "Transaction {} not found",
                self.tx_id
            )))
        })
    }

    fn validate(
        &self,
        _state: &AccountState,
        resource: &Self::Resource,
    ) -> Result<Self::Entity, PaymentError> {
        match resource {
            TransactionTypeEvent::Deposited(_) | TransactionTypeEvent::Withdrawn(_) => {}
            _ => {
                return Err(PaymentError::Transaction(
                    TransactionError::InvalidTransactionType,
                ));
            }
        }

        // Disputes are allowed even on frozen accounts for consumer protection.
        // Clients should be able to dispute fraudulent transactions regardless of account status.
        Ok(())
    }

    fn emit(
        &self,
        _state: &AccountState,
        _entity: &Self::Entity,
        resource: &Self::Resource,
        _timestamp: DateTime<Utc>,
    ) -> Result<Vec<TransactionTypeEvent>, PaymentError> {
        let amount = match resource {
            TransactionTypeEvent::Deposited(d) => d.amount,
            TransactionTypeEvent::Withdrawn(w) => w.amount,
            _ => {
                return Err(PaymentError::Transaction(
                    TransactionError::InvalidTransactionType,
                ));
            }
        };

        Ok(vec![TransactionTypeEvent::Disputed(Disputed {
            client_id: self.client_id,
            tx_id: self.tx_id,
            amount,
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
