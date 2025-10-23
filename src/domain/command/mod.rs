use serde::{Deserialize, Serialize};

/// CSV row structure (flat deserialization)
#[derive(Debug, Deserialize)]
struct CsvRow {
    #[serde(rename = "type")]
    transaction_type: String,
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(rename = "tx")]
    tx_id: u32,
    #[serde(default)]
    amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
/// A transaction is a single action on the client's account. It can be a deposit, withdrawal, dispute, resolve, or chargeback.
///
/// In this event sourcing implementation, the TransactionType is the representation of Command,
/// each command will then be persisted and applied to the AccountState to build the current state of the account.
pub enum TransactionTypeCommand {
    Deposit(Deposit),
    Withdrawal(Withdraw),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}

// Custom Deserialize implementation for CSV format
impl<'de> Deserialize<'de> for TransactionTypeCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let row = CsvRow::deserialize(deserializer)?;
        row.try_into().map_err(serde::de::Error::custom)
    }
}

impl TryFrom<CsvRow> for TransactionTypeCommand {
    type Error = String;

    fn try_from(row: CsvRow) -> Result<Self, Self::Error> {
        match row.transaction_type.to_lowercase().as_str() {
            "deposit" => {
                let amount = row
                    .amount
                    .ok_or_else(|| "deposit requires amount".to_string())?;
                Ok(Self::Deposit(Deposit {
                    client_id: row.client_id,
                    tx_id: row.tx_id,
                    amount,
                }))
            }
            "withdrawal" => {
                let amount = row
                    .amount
                    .ok_or_else(|| "withdrawal requires amount".to_string())?;
                Ok(Self::Withdrawal(Withdraw {
                    client_id: row.client_id,
                    tx_id: row.tx_id,
                    amount,
                }))
            }
            "dispute" => Ok(Self::Dispute(Dispute {
                client_id: row.client_id,
                tx_id: row.tx_id,
            })),
            "resolve" => Ok(Self::Resolve(Resolve {
                client_id: row.client_id,
                tx_id: row.tx_id,
            })),
            "chargeback" => Ok(Self::Chargeback(Chargeback {
                client_id: row.client_id,
                tx_id: row.tx_id,
            })),
            other => Err(format!("unknown transaction type: {}", other)),
        }
    }
}

// These two should probably be like a required trait such that it can be called abstractly
// by the engine.
impl TransactionTypeCommand {
    pub fn client_id(&self) -> u16 {
        match self {
            TransactionTypeCommand::Deposit(cmd) => cmd.client_id,
            TransactionTypeCommand::Withdrawal(cmd) => cmd.client_id,
            TransactionTypeCommand::Dispute(cmd) => cmd.client_id,
            TransactionTypeCommand::Resolve(cmd) => cmd.client_id,
            TransactionTypeCommand::Chargeback(cmd) => cmd.client_id,
        }
    }

    pub fn tx_id(&self) -> u32 {
        match self {
            TransactionTypeCommand::Deposit(cmd) => cmd.tx_id,
            TransactionTypeCommand::Withdrawal(cmd) => cmd.tx_id,
            TransactionTypeCommand::Dispute(cmd) => cmd.tx_id,
            TransactionTypeCommand::Resolve(cmd) => cmd.tx_id,
            TransactionTypeCommand::Chargeback(cmd) => cmd.tx_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A chargeback is the final state of a dispute and represents the client reversing a transaction.
/// Funds that were held have now been withdrawn. This means that the clients held funds and total
/// funds should decrease by the amount previously disputed. If a chargeback occurs the client's
/// account should be immediately frozen.
///
/// Like a dispute and a resolve a chargeback refers to the transaction by ID (tx) and does not
/// specify an amount. Like a resolve, if the tx specified doesn't exist, or the tx isn't under dispute,
/// you can ignore chargeback and assume this is an error on our partner's side.
pub struct Chargeback {
    pub client_id: u16,
    pub tx_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A deposit is a credit to the client's asset account, meaning it should increase the available and
/// total funds of the client account
pub struct Deposit {
    pub client_id: u16,
    pub tx_id: u32,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A dispute represents a client's claim that a transaction was erroneous and should be reversed.
///
/// The transaction shouldn't be reversed yet but the associated funds should be held. This means
/// that the clients available funds should decrease by the amount disputed, their held funds should
/// increase by the amount disputed, while their total funds should remain the same.
///
/// Notice that a dispute does not state the amount disputed. Instead a dispute references the
/// transaction that is disputed by ID. If the tx specified by the dispute doesn't exist you can ignore it
/// and assume this is an error on our partners side.
pub struct Dispute {
    pub client_id: u16,
    pub tx_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A resolve represents a resolution to a dispute, releasing the associated held funds. Funds that
/// were previously disputed are no longer disputed. This means that the clients held funds should
/// decrease by the amount no longer disputed, their available funds should increase by the amount
/// no longer disputed, and their total funds should remain the same.
///
/// Like disputes, resolves do not specify an amount. Instead they refer to a transaction that was
/// under dispute by ID. If the tx specified doesn't exist, or the tx isn't under dispute, you can ignore
/// the resolve and assume this is an error on our partner's side.
pub struct Resolve {
    pub client_id: u16,
    pub tx_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A withdraw is a debit to the client's asset account, meaning it should decrease the available and
/// total funds of the client account
///
/// If a client does not have sufficient available funds the withdrawal should fail and the total amount
/// of funds should not change
pub struct Withdraw {
    pub client_id: u16,
    pub tx_id: u32,
    pub amount: f64,
}
