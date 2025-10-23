use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", untagged)]
pub enum AccountState {
    Active(ActiveAccountState),
    Frozen(FrozenAccountState),
}

/// Active account state - only balances (O(1) memory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAccountState {
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub last_activity: DateTime<Utc>,
}

/// Frozen account state - only balances (O(1) memory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenAccountState {
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub last_activity: DateTime<Utc>,
}

// It is my prerrogative to assume that Frozen and Active may diverge in the future,
// so I'm keeping them separate even though they've the exact same structure for now.
