use crate::{
    domain::{AccountState, PaymentError, TransactionTypeCommand, TransactionTypeEvent},
    port::TransactionLookup,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait CommandHandler {
    type Resource;
    type Entity;

    /// Load resources required to process the command
    ///
    /// This runs CONCURRENTLY with potentially stale state (fast-moving state is OK).
    /// Can be slow - do DB queries, HTTP calls, etc.
    async fn load(
        &self,
        stale_state: &AccountState,
        lookup: &dyn TransactionLookup,
    ) -> Result<Self::Resource, PaymentError>;

    /// Validate command against ACTUAL state
    ///
    /// This runs with EXCLUSIVE ACCESS to actual state - MUST BE FAST!
    /// No async, no I/O, just pure business logic.
    fn validate(
        &self,
        actual_state: &AccountState,
        resource: &Self::Resource,
    ) -> Result<Self::Entity, PaymentError>;

    /// Emit events from validated entity
    ///
    /// MUST BE FAST - no async, no I/O.
    /// Just creates events from the validated entity.
    /// Returns a Vec to support multiple events per command.
    fn emit(
        &self,
        state: &AccountState,
        entity: &Self::Entity,
        resource: &Self::Resource,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<TransactionTypeEvent>, PaymentError>;

    /// Execute side effects after event is persisted
    ///
    /// Can be slow - happens after persistence and state update
    async fn effect(
        &self,
        previous_state: &AccountState,
        state: &AccountState,
        resource: &Self::Resource,
        entity: &Self::Entity,
        timestamp: DateTime<Utc>,
    ) -> Result<(), PaymentError>;
}

#[async_trait]
pub trait CommandProcessor {
    type Context;

    async fn process(
        &self,
        command: TransactionTypeCommand,
        previous_state: &AccountState,
        context: &mut Self::Context,
    ) -> Result<AccountState, PaymentError>;
}
