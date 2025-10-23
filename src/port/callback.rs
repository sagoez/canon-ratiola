use crate::domain::*;
use crate::port::Journal;
use async_trait::async_trait;
use std::sync::Arc;

/// Context provided to event callbacks
pub struct CallbackContext {
    /// The journal - for infrastructure callbacks to update indices
    pub journal: Arc<dyn Journal + Send + Sync>,
    /// The persisted event envelope (includes sequence number, timestamp, etc.)
    pub envelope: EventEnvelope,
}

/// Infrastructure callbacks invoked after events are persisted
///
/// Implementations can maintain indices, caches, or other infrastructure concerns.
/// These are called by the Engine after successful event persistence.
///
/// Callbacks receive the CallbackContext which includes:
/// - journal: for updating infrastructure (indices, caches)
/// - envelope: the persisted event with metadata
#[async_trait]
pub trait EventCallback: Send + Sync {
    /// Called after a Deposited event is persisted
    async fn on_deposited(
        &self,
        event: &Deposited,
        ctx: &CallbackContext,
    ) -> Result<(), PaymentError> {
        let _ = (event, ctx);
        Ok(())
    }

    /// Called after a Withdrawn event is persisted
    async fn on_withdrawn(
        &self,
        event: &Withdrawn,
        ctx: &CallbackContext,
    ) -> Result<(), PaymentError> {
        let _ = (event, ctx);
        Ok(())
    }

    /// Called after a Disputed event is persisted
    async fn on_disputed(
        &self,
        event: &Disputed,
        ctx: &CallbackContext,
    ) -> Result<(), PaymentError> {
        let _ = (event, ctx);
        Ok(())
    }

    /// Called after a Resolved event is persisted
    async fn on_resolved(
        &self,
        event: &Resolved,
        ctx: &CallbackContext,
    ) -> Result<(), PaymentError> {
        let _ = (event, ctx);
        Ok(())
    }

    /// Called after a Chargebacked event is persisted
    async fn on_chargebacked(
        &self,
        event: &Chargebacked,
        ctx: &CallbackContext,
    ) -> Result<(), PaymentError> {
        let _ = (event, ctx);
        Ok(())
    }
}
