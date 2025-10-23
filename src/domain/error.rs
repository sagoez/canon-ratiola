use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use thiserror::Error;

// Think about the possibility of adding more error if needed:
//
// Will I need amount errors?
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TransactionError {
    #[error("Insufficient funds for transaction")]
    InsufficientFunds,
    #[error("Account is locked")]
    AccountLocked,
    #[error("Transaction not found")]
    TransactionNotFound,
    #[error("Duplicate transaction ID")]
    DuplicateTransaction,
    #[error("Invalid transaction type")]
    InvalidTransactionType,
    #[error("Invalid amount (must be positive)")]
    InvalidAmount,
    #[error("General transaction error: {0}")]
    GeneralError(String),
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum EngineError {
    #[error("Loading resources error: {0}")]
    LoadingResourcesError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Emitting event error: {0}")]
    EmittingEventError(String),
    #[error("Effecting command error: {0}")]
    SideEffectError(String),
    #[error("No events produced by command handler")]
    NoEvents,
    #[error("State transition failed - event could not be applied")]
    StateTransitionFailed,
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PaymentError {
    Engine(EngineError),
    Transaction(TransactionError),
}

impl Display for PaymentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentError::Engine(e) => e.fmt(f),
            PaymentError::Transaction(e) => e.fmt(f),
        }
    }
}
