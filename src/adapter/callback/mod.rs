use crate::domain::*;
use crate::port::{CallbackContext, DisputeIndex, EventCallback};
use async_trait::async_trait;
use std::sync::Arc;

/// Callback adapter: forwards events to DisputeIndex trait methods
///
/// This bridges EventCallback (engine concern) to DisputeIndex (infrastructure port)
pub struct DisputeIndexCallback {
    index: Arc<dyn DisputeIndex>,
}

impl DisputeIndexCallback {
    pub fn new(index: Arc<dyn DisputeIndex>) -> Self {
        Self { index }
    }
}

#[async_trait]
impl EventCallback for DisputeIndexCallback {
    async fn on_disputed(
        &self,
        event: &Disputed,
        _ctx: &CallbackContext,
    ) -> Result<(), PaymentError> {
        self.index.mark_disputed(event.tx_id, event.amount).await
    }

    async fn on_resolved(
        &self,
        event: &Resolved,
        _ctx: &CallbackContext,
    ) -> Result<(), PaymentError> {
        self.index.unmark_disputed(event.tx_id).await
    }

    async fn on_chargebacked(
        &self,
        event: &Chargebacked,
        _ctx: &CallbackContext,
    ) -> Result<(), PaymentError> {
        self.index.unmark_disputed(event.tx_id).await
    }
}
