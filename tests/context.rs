/// Shared test utilities and helpers
use payment::{
    adapter::{
        CommandProcessor, DisputeIndexCallback, EngineContext, InMemoryDisputeIndex,
        InMemoryJournal, JournalTransactionLookup, PaymentEngine,
    },
    domain::{
        AccountState, ActiveAccountState, CommandMetadata, DeduplicationKey, PaymentError,
        TransactionTypeCommand,
    },
    port::{DisputeIndex, Engine},
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Test context that provides a complete payment engine setup
pub struct TestContext {
    pub journal: Arc<InMemoryJournal>,
    pub engine: Arc<PaymentEngine>,
    pub account_state: AccountState,
}

impl TestContext {
    /// Create a new test context with empty account state
    pub fn new() -> Self {
        let journal = Arc::new(InMemoryJournal::new());
        let dispute_index: Arc<dyn DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());
        let lookup = Arc::new(JournalTransactionLookup::new(
            journal.clone(),
            dispute_index.clone(),
        ));
        let processor = Arc::new(CommandProcessor::new(lookup));

        let dispute_callback = Arc::new(DisputeIndexCallback::new(dispute_index));
        let engine = Arc::new(PaymentEngine::new(processor).with_callback(dispute_callback));

        let account_state = AccountState::Active(ActiveAccountState {
            available: 0.0,
            held: 0.0,
            total: 0.0,
            last_activity: chrono::Utc::now(),
        });

        Self {
            journal,
            engine,
            account_state,
        }
    }

    /// Process a command and update the account state
    pub async fn process(
        &mut self,
        command: TransactionTypeCommand,
        client_id: u16,
    ) -> Result<(), PaymentError> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let command_id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dedup_key = format!("test:{}:{}", client_id, command_id);

        let metadata = CommandMetadata {
            deduplication_key: DeduplicationKey::new(dedup_key),
        };

        let context = EngineContext {
            journal: self.journal.clone(),
            current_state: self.account_state.clone(),
        };

        let (_envelope, new_state) = self
            .engine
            .process_command(command, metadata, &context)
            .await?;
        self.account_state = new_state;
        Ok(())
    }

    /// Get available balance
    pub fn available(&self) -> f64 {
        match &self.account_state {
            AccountState::Active(state) => state.available,
            AccountState::Frozen(state) => state.available,
        }
    }

    /// Get held balance
    pub fn held(&self) -> f64 {
        match &self.account_state {
            AccountState::Active(state) => state.held,
            AccountState::Frozen(state) => state.held,
        }
    }

    /// Get total balance
    pub fn total(&self) -> f64 {
        match &self.account_state {
            AccountState::Active(state) => state.total,
            AccountState::Frozen(state) => state.total,
        }
    }

    /// Check if account is frozen
    pub fn is_frozen(&self) -> bool {
        matches!(self.account_state, AccountState::Frozen(_))
    }

    /// Assert balances match expected values
    pub fn assert_balances(&self, available: f64, held: f64, total: f64) {
        assert_eq!(self.available(), available, "Available balance mismatch");
        assert_eq!(self.held(), held, "Held balance mismatch");
        assert_eq!(self.total(), total, "Total balance mismatch");
    }
}

/// Helper to create a deposit command
pub fn deposit(client: u16, tx: u32, amount: f64) -> TransactionTypeCommand {
    use payment::domain::Deposit;
    TransactionTypeCommand::Deposit(Deposit {
        client_id: client,
        tx_id: tx,
        amount,
    })
}

/// Helper to create a withdrawal command
pub fn withdrawal(client: u16, tx: u32, amount: f64) -> TransactionTypeCommand {
    use payment::domain::Withdraw;
    TransactionTypeCommand::Withdrawal(Withdraw {
        client_id: client,
        tx_id: tx,
        amount,
    })
}

/// Helper to create a dispute command
pub fn dispute(client: u16, tx: u32) -> TransactionTypeCommand {
    use payment::domain::Dispute;
    TransactionTypeCommand::Dispute(Dispute {
        client_id: client,
        tx_id: tx,
    })
}

/// Helper to create a resolve command
pub fn resolve(client: u16, tx: u32) -> TransactionTypeCommand {
    use payment::domain::Resolve;
    TransactionTypeCommand::Resolve(Resolve {
        client_id: client,
        tx_id: tx,
    })
}

/// Helper to create a chargeback command
pub fn chargeback(client: u16, tx: u32) -> TransactionTypeCommand {
    use payment::domain::Chargeback;
    TransactionTypeCommand::Chargeback(Chargeback {
        client_id: client,
        tx_id: tx,
    })
}

/// Assert that processing a command fails
#[macro_export]
macro_rules! assert_fails {
    ($ctx:expr, $cmd:expr) => {
        assert!(
            $ctx.process($cmd, 1).await.is_err(),
            "Expected command to fail but it succeeded"
        );
    };
}

/// Assert that processing a command succeeds
#[macro_export]
macro_rules! assert_succeeds {
    ($ctx:expr, $cmd:expr) => {
        $ctx.process($cmd, 1)
            .await
            .expect("Expected command to succeed but it failed");
    };
}
