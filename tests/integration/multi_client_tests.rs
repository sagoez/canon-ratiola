use payment::adapter::{ClientRegistry, InMemoryDisputeIndex, InMemoryJournal};
use payment::domain::*;
use payment::port::{DisputeIndex, Journal};
use std::sync::Arc;

#[tokio::test]
async fn test_multiple_clients_independent_balances() {
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());
    let dispute_index: Arc<dyn DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());
    let registry = ClientRegistry::with_namespace(
        journal,
        dispute_index,
        format!("test-{}", uuid::Uuid::new_v4()),
    );

    // Client 1: Deposit 100
    let deposit1 = Deposit {
        client_id: 1,
        tx_id: 1,
        amount: 100.0,
    };

    let metadata1 = CommandMetadata {
        deduplication_key: DeduplicationKey::new("test:1:1".to_string()),
    };

    registry
        .process_command(1, TransactionTypeCommand::Deposit(deposit1), metadata1)
        .await
        .unwrap();

    // Client 2: Deposit 200
    let deposit2 = Deposit {
        client_id: 2,
        tx_id: 2,
        amount: 200.0,
    };

    let metadata2 = CommandMetadata {
        deduplication_key: DeduplicationKey::new("test:2:2".to_string()),
    };

    registry
        .process_command(2, TransactionTypeCommand::Deposit(deposit2), metadata2)
        .await
        .unwrap();

    // Verify balances
    let state1 = registry.get_state(1).await.unwrap().unwrap();
    let state2 = registry.get_state(2).await.unwrap().unwrap();

    match state1 {
        AccountState::Active(active) => {
            assert_eq!(active.available, 100.0);
            assert_eq!(active.total, 100.0);
        }
        _ => panic!("Expected Active state for client 1"),
    }

    match state2 {
        AccountState::Active(active) => {
            assert_eq!(active.available, 200.0);
            assert_eq!(active.total, 200.0);
        }
        _ => panic!("Expected Active state for client 2"),
    }
}

#[tokio::test]
async fn test_concurrent_client_operations() {
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());
    let dispute_index: Arc<dyn DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());
    let registry = ClientRegistry::with_namespace(
        journal,
        dispute_index,
        format!("test-{}", uuid::Uuid::new_v4()),
    );

    // Spawn multiple clients concurrently
    let handles: Vec<_> = (1..=10)
        .map(|client_id| {
            let registry_clone = registry.clone();
            tokio::spawn(async move {
                // Each client deposits 100
                let deposit = Deposit {
                    client_id,
                    tx_id: client_id as u32,
                    amount: 100.0,
                };

                let metadata = CommandMetadata {
                    deduplication_key: DeduplicationKey::new(format!(
                        "test:{}:{}",
                        client_id, client_id
                    )),
                };

                registry_clone
                    .process_command(
                        client_id,
                        TransactionTypeCommand::Deposit(deposit),
                        metadata,
                    )
                    .await
                    .unwrap();

                client_id
            })
        })
        .collect();

    // Wait for all clients
    let mut client_ids = Vec::new();
    for handle in handles {
        let client_id = handle.await.unwrap();
        client_ids.push(client_id);
    }

    // Verify each client
    for client_id in client_ids {
        let state = registry.get_state(client_id).await.unwrap().unwrap();

        match state {
            AccountState::Active(active) => {
                assert_eq!(active.available, 100.0);
                assert_eq!(active.total, 100.0);
            }
            _ => panic!("Expected Active state"),
        }
    }
}

#[tokio::test]
async fn test_client_isolation_disputes_dont_affect_others() {
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());
    let dispute_index: Arc<dyn DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());
    let registry = ClientRegistry::with_namespace(
        journal,
        dispute_index,
        format!("test-{}", uuid::Uuid::new_v4()),
    );

    // Client 1: Deposit and dispute
    let deposit1 = Deposit {
        client_id: 1,
        tx_id: 1,
        amount: 100.0,
    };
    let metadata1 = CommandMetadata {
        deduplication_key: DeduplicationKey::new("test:1:1".to_string()),
    };
    registry
        .process_command(1, TransactionTypeCommand::Deposit(deposit1), metadata1)
        .await
        .unwrap();

    let dispute1 = Dispute {
        client_id: 1,
        tx_id: 1,
    };
    let metadata2 = CommandMetadata {
        deduplication_key: DeduplicationKey::new("test:1:dispute".to_string()),
    };
    registry
        .process_command(1, TransactionTypeCommand::Dispute(dispute1), metadata2)
        .await
        .unwrap();

    // Client 2: Just deposit
    let deposit2 = Deposit {
        client_id: 2,
        tx_id: 2,
        amount: 200.0,
    };
    let metadata3 = CommandMetadata {
        deduplication_key: DeduplicationKey::new("test:2:2".to_string()),
    };
    registry
        .process_command(2, TransactionTypeCommand::Deposit(deposit2), metadata3)
        .await
        .unwrap();

    // Verify client 1 has disputed funds
    let state1 = registry.get_state(1).await.unwrap().unwrap();
    match state1 {
        AccountState::Active(active) => {
            assert_eq!(active.available, 0.0);
            assert_eq!(active.held, 100.0);
            assert_eq!(active.total, 100.0);
        }
        _ => panic!("Expected Active state for client 1"),
    }

    // Verify client 2 is unaffected
    let state2 = registry.get_state(2).await.unwrap().unwrap();
    match state2 {
        AccountState::Active(active) => {
            assert_eq!(active.available, 200.0);
            assert_eq!(active.held, 0.0);
            assert_eq!(active.total, 200.0);
        }
        _ => panic!("Expected Active state for client 2"),
    }
}

#[tokio::test]
async fn test_same_tx_id_different_clients() {
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());
    let dispute_index: Arc<dyn DisputeIndex> = Arc::new(InMemoryDisputeIndex::new());
    let registry = ClientRegistry::with_namespace(
        journal,
        dispute_index,
        format!("test-{}", uuid::Uuid::new_v4()),
    );

    // Both clients use tx_id 1 (should be fine, as they're in different namespaces)
    let deposit1 = Deposit {
        client_id: 1,
        tx_id: 1,
        amount: 100.0,
    };
    let metadata1 = CommandMetadata {
        deduplication_key: DeduplicationKey::new("test:1:1".to_string()),
    };
    registry
        .process_command(1, TransactionTypeCommand::Deposit(deposit1), metadata1)
        .await
        .unwrap();

    let deposit2 = Deposit {
        client_id: 2,
        tx_id: 1, // Same tx_id as client 1
        amount: 200.0,
    };
    let metadata2 = CommandMetadata {
        deduplication_key: DeduplicationKey::new("test:2:1".to_string()),
    };
    registry
        .process_command(2, TransactionTypeCommand::Deposit(deposit2), metadata2)
        .await
        .unwrap();

    // Both should succeed
    let state1 = registry.get_state(1).await.unwrap().unwrap();
    let state2 = registry.get_state(2).await.unwrap().unwrap();

    match state1 {
        AccountState::Active(active) => assert_eq!(active.total, 100.0),
        _ => panic!("Expected Active state for client 1"),
    }

    match state2 {
        AccountState::Active(active) => assert_eq!(active.total, 200.0),
        _ => panic!("Expected Active state for client 2"),
    }
}
