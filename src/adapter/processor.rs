use crate::{
    domain::{AccountState, Directive, PaymentError, TransactionTypeCommand},
    port::{CommandHandler, EffectFn, Processor, TransactionLookup, ValidateFn},
};
use async_trait::async_trait;
use chrono::Utc;

use std::sync::Arc;

/// CommandProcessor dispatches commands to their handlers
pub struct CommandProcessor {
    lookup: Arc<dyn TransactionLookup>,
}

impl CommandProcessor {
    pub fn new(lookup: Arc<dyn TransactionLookup>) -> Self {
        Self { lookup }
    }

    pub fn lookup(&self) -> &Arc<dyn TransactionLookup> {
        &self.lookup
    }
}

#[async_trait]
impl Processor for CommandProcessor {
    async fn load(
        &self,
        command: TransactionTypeCommand,
        stale_state: &AccountState,
    ) -> Result<Box<dyn ValidateFn>, PaymentError> {
        match command {
            TransactionTypeCommand::Deposit(cmd) => {
                let resource = cmd.load(stale_state, self.lookup.as_ref()).await?;
                Ok(Box::new(LoadedCommand::new(cmd, resource)))
            }
            TransactionTypeCommand::Withdrawal(cmd) => {
                let resource = cmd.load(stale_state, self.lookup.as_ref()).await?;
                Ok(Box::new(LoadedCommand::new(cmd, resource)))
            }
            TransactionTypeCommand::Dispute(cmd) => {
                let resource = cmd.load(stale_state, self.lookup.as_ref()).await?;
                Ok(Box::new(LoadedCommand::new(cmd, resource)))
            }
            TransactionTypeCommand::Resolve(cmd) => {
                let resource = cmd.load(stale_state, self.lookup.as_ref()).await?;
                Ok(Box::new(LoadedCommand::new(cmd, resource)))
            }
            TransactionTypeCommand::Chargeback(cmd) => {
                let resource = cmd.load(stale_state, self.lookup.as_ref()).await?;
                Ok(Box::new(LoadedCommand::new(cmd, resource)))
            }
        }
    }
}

struct LoadedCommand<H: CommandHandler> {
    handler: H,
    resource: H::Resource,
}

impl<H: CommandHandler> LoadedCommand<H> {
    fn new(handler: H, resource: H::Resource) -> Self {
        Self { handler, resource }
    }
}

impl<H> ValidateFn for LoadedCommand<H>
where
    H: CommandHandler + Clone + Send + Sync + 'static,
    H::Resource: Clone + Send + Sync + 'static,
    H::Entity: Clone + Send + Sync + 'static,
{
    fn apply(&self, actual_state: &AccountState) -> Result<Directive, PaymentError> {
        let entity = self.handler.validate(actual_state, &self.resource)?;

        let events = self
            .handler
            .emit(actual_state, &entity, &self.resource, Utc::now())?;

        let handler = self.handler.clone();
        let resource = self.resource.clone();
        let entity = entity.clone();
        let previous_state = actual_state.clone();

        let effects: Vec<Box<dyn EffectFn>> = vec![Box::new(CommandEffect {
            handler,
            resource,
            entity,
            previous_state,
        })];

        Ok(Directive { events, effects })
    }
}

struct CommandEffect<H: CommandHandler> {
    handler: H,
    resource: H::Resource,
    entity: H::Entity,
    previous_state: AccountState,
}

#[async_trait]
impl<H> EffectFn for CommandEffect<H>
where
    H: CommandHandler + Send + Sync,
    H::Resource: Send + Sync,
    H::Entity: Send + Sync,
{
    async fn execute(&self, new_state: &AccountState) -> Result<(), PaymentError> {
        self.handler
            .effect(
                &self.previous_state,
                new_state,
                &self.resource,
                &self.entity,
                Utc::now(),
            )
            .await
    }
}
