use payment::adapter::{ClientRegistry, InMemoryDisputeIndex, InMemoryJournal};
use payment::domain::OrchestratorMode;
use payment::port::{DisputeIndex, Journal};
use payment::service::Orchestrator;
use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_csv_processing_single_client() {
    // Create a temporary CSV file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "type,client,tx,amount").unwrap();
    writeln!(temp_file, "deposit,1,1,100.0").unwrap();
    writeln!(temp_file, "deposit,1,2,200.0").unwrap();
    writeln!(temp_file, "withdrawal,1,3,50.0").unwrap();
    temp_file.flush().unwrap();

    // Create registry with unique namespace
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());
    let dispute_index: Arc<dyn DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());
    let registry = ClientRegistry::with_namespace(
        journal,
        dispute_index,
        format!("test-{}", uuid::Uuid::new_v4()),
    );

    // Process CSV
    let orchestrator = Orchestrator::with_registry(
        registry,
        OrchestratorMode::Csv {
            file_path: temp_file.path().to_str().unwrap().to_string(),
        },
    );

    let states = orchestrator.process().await.unwrap();

    // Verify final state
    let state = states.get(&1).unwrap();

    match state {
        payment::domain::AccountState::Active(active) => {
            assert_eq!(active.available, 250.0);
            assert_eq!(active.total, 250.0);
        }
        _ => panic!("Expected Active state"),
    }
}

#[tokio::test]
async fn test_csv_processing_multiple_clients() {
    // Create a temporary CSV file with multiple clients
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "type,client,tx,amount").unwrap();
    writeln!(temp_file, "deposit,1,1,100.0").unwrap();
    writeln!(temp_file, "deposit,2,2,200.0").unwrap();
    writeln!(temp_file, "deposit,3,3,300.0").unwrap();
    writeln!(temp_file, "withdrawal,1,4,10.0").unwrap();
    writeln!(temp_file, "withdrawal,2,5,20.0").unwrap();
    temp_file.flush().unwrap();

    // Create registry with unique namespace
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());
    let dispute_index: Arc<dyn DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());
    let registry = ClientRegistry::with_namespace(
        journal,
        dispute_index,
        format!("test-{}", uuid::Uuid::new_v4()),
    );

    // Process CSV
    let orchestrator = Orchestrator::with_registry(
        registry,
        OrchestratorMode::Csv {
            file_path: temp_file.path().to_str().unwrap().to_string(),
        },
    );

    let states = orchestrator.process().await.unwrap();

    // Verify all clients
    let state1 = states.get(&1).unwrap();
    match state1 {
        payment::domain::AccountState::Active(active) => {
            assert_eq!(active.available, 90.0);
        }
        _ => panic!("Expected Active state"),
    }

    let state2 = states.get(&2).unwrap();
    match state2 {
        payment::domain::AccountState::Active(active) => {
            assert_eq!(active.available, 180.0);
        }
        _ => panic!("Expected Active state"),
    }

    let state3 = states.get(&3).unwrap();
    match state3 {
        payment::domain::AccountState::Active(active) => {
            assert_eq!(active.available, 300.0);
        }
        _ => panic!("Expected Active state"),
    }
}

#[tokio::test]
async fn test_csv_processing_with_disputes() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "type,client,tx,amount").unwrap();
    writeln!(temp_file, "deposit,1,1,100.0").unwrap();
    writeln!(temp_file, "deposit,1,2,50.0").unwrap();
    writeln!(temp_file, "dispute,1,1,").unwrap();
    writeln!(temp_file, "resolve,1,1,").unwrap();
    temp_file.flush().unwrap();

    // Create registry with unique namespace
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());
    let dispute_index: Arc<dyn DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());
    let registry = ClientRegistry::with_namespace(
        journal,
        dispute_index,
        format!("test-{}", uuid::Uuid::new_v4()),
    );

    let orchestrator = Orchestrator::with_registry(
        registry,
        OrchestratorMode::Csv {
            file_path: temp_file.path().to_str().unwrap().to_string(),
        },
    );

    let states = orchestrator.process().await.unwrap();

    let state = states.get(&1).unwrap();

    match state {
        payment::domain::AccountState::Active(active) => {
            assert_eq!(active.available, 150.0);
            assert_eq!(active.held, 0.0);
            assert_eq!(active.total, 150.0);
        }
        _ => panic!("Expected Active state"),
    }
}

#[tokio::test]
async fn test_csv_processing_with_chargeback() {
    // Create a temporary CSV file with chargeback
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "type,client,tx,amount").unwrap();
    writeln!(temp_file, "deposit,1,1,100.0").unwrap();
    writeln!(temp_file, "deposit,1,2,50.0").unwrap();
    writeln!(temp_file, "dispute,1,1,").unwrap();
    writeln!(temp_file, "chargeback,1,1,").unwrap();
    temp_file.flush().unwrap();

    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());
    let dispute_index: Arc<dyn DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());
    let registry = ClientRegistry::with_namespace(
        journal,
        dispute_index,
        format!("test-{}", uuid::Uuid::new_v4()),
    );

    let orchestrator = Orchestrator::with_registry(
        registry,
        OrchestratorMode::Csv {
            file_path: temp_file.path().to_str().unwrap().to_string(),
        },
    );

    let states = orchestrator.process().await.unwrap();

    let state = states.get(&1).unwrap();

    match state {
        payment::domain::AccountState::Frozen(frozen) => {
            assert_eq!(frozen.available, 50.0);
            assert_eq!(frozen.held, 0.0);
            assert_eq!(frozen.total, 50.0);
        }
        _ => panic!("Expected Frozen state"),
    }
}

#[tokio::test]
async fn test_csv_processing_invalid_withdrawal() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "type,client,tx,amount").unwrap();
    writeln!(temp_file, "deposit,1,1,50.0").unwrap();
    writeln!(temp_file, "withdrawal,1,2,100.0").unwrap(); // Insufficient funds
    writeln!(temp_file, "deposit,1,3,25.0").unwrap();
    temp_file.flush().unwrap();

    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());
    let dispute_index: Arc<dyn DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());
    let registry = ClientRegistry::with_namespace(
        journal,
        dispute_index,
        format!("test-{}", uuid::Uuid::new_v4()),
    );

    let orchestrator = Orchestrator::with_registry(
        registry,
        OrchestratorMode::Csv {
            file_path: temp_file.path().to_str().unwrap().to_string(),
        },
    );

    let states = orchestrator.process().await.unwrap();

    let state = states.get(&1).unwrap();

    match state {
        payment::domain::AccountState::Active(active) => {
            assert_eq!(active.available, 75.0); // Only deposits succeeded
        }
        _ => panic!("Expected Active state"),
    }
}
