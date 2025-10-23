use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionTypeEvent {
    Deposited(Deposited),
    Withdrawn(Withdrawn),
    Disputed(Disputed),
    Resolved(Resolved),
    Chargebacked(Chargebacked),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chargebacked {
    pub client_id: u16,
    pub tx_id: u32,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposited {
    pub client_id: u16,
    pub tx_id: u32,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disputed {
    pub client_id: u16,
    pub tx_id: u32,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolved {
    pub client_id: u16,
    pub tx_id: u32,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Withdrawn {
    pub client_id: u16,
    pub tx_id: u32,
    pub amount: f64,
}
