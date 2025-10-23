use payment::adapter::InMemoryJournal;
use payment::domain::*;
use payment::port::Journal;
use std::sync::Arc;

#[tokio::test]
async fn test_duplicate_deduplication_key_returns_existing_event() {
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());

    let dedup_key = DeduplicationKey::new("deposit:1:1".to_string());

    let metadata1 = EventMetadata {
        client_id: 1,
        tx_id: 1,
        timestamp: chrono::Utc::now(),
        deduplication_key: dedup_key.clone(),
    };

    let envelope1 = journal
        .append(
            TransactionTypeEvent::Deposited(Deposited {
                client_id: 1,
                tx_id: 1,
                amount: 100.0,
            }),
            metadata1,
        )
        .await
        .unwrap();

    let metadata2 = EventMetadata {
        client_id: 1,
        tx_id: 1,
        timestamp: chrono::Utc::now(),
        deduplication_key: dedup_key.clone(),
    };

    let envelope2 = journal
        .append(
            TransactionTypeEvent::Deposited(Deposited {
                client_id: 1,
                tx_id: 1,
                amount: 200.0, // Different amount, but should return original
            }),
            metadata2,
        )
        .await
        .unwrap();

    assert_eq!(envelope1.sequence_nr, envelope2.sequence_nr);
    assert_eq!(envelope1.tx_id, envelope2.tx_id);

    match (&envelope1.event, &envelope2.event) {
        (TransactionTypeEvent::Deposited(d1), TransactionTypeEvent::Deposited(d2)) => {
            assert_eq!(d1.amount, 100.0);
            assert_eq!(d2.amount, 100.0); // Original amount, not 200.0
        }
        _ => panic!("Expected Deposited events"),
    }

    // Journal should only have one event
    let events = journal.replay(None).await.unwrap();
    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn test_different_transaction_types_with_same_tx_id() {
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());

    let metadata1 = EventMetadata {
        client_id: 1,
        tx_id: 1,
        timestamp: chrono::Utc::now(),
        deduplication_key: DeduplicationKey::new("deposit:1:1".to_string()),
    };

    journal
        .append(
            TransactionTypeEvent::Deposited(Deposited {
                client_id: 1,
                tx_id: 1,
                amount: 100.0,
            }),
            metadata1,
        )
        .await
        .unwrap();

    let metadata2 = EventMetadata {
        client_id: 1,
        tx_id: 1,
        timestamp: chrono::Utc::now(),
        deduplication_key: DeduplicationKey::new("dispute:1:1".to_string()),
    };

    let result = journal
        .append(
            TransactionTypeEvent::Disputed(Disputed {
                client_id: 1,
                tx_id: 1,
                amount: 100.0,
            }),
            metadata2,
        )
        .await;

    assert!(
        result.is_ok(),
        "Should allow different transaction types with same tx_id"
    );

    // Should have two events
    let events = journal.replay(None).await.unwrap();
    assert_eq!(events.len(), 2);
}

#[tokio::test]
async fn test_concurrent_appends_with_different_keys() {
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());

    let handles: Vec<_> = (1..=10)
        .map(|i| {
            let journal_clone = journal.clone();
            tokio::spawn(async move {
                let metadata = EventMetadata {
                    client_id: 1,
                    tx_id: i,
                    timestamp: chrono::Utc::now(),
                    deduplication_key: DeduplicationKey::new(format!("deposit:1:{}", i)),
                };

                journal_clone
                    .append(
                        TransactionTypeEvent::Deposited(Deposited {
                            client_id: 1,
                            tx_id: i,
                            amount: 10.0,
                        }),
                        metadata,
                    )
                    .await
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    let events = journal.replay(None).await.unwrap();
    assert_eq!(events.len(), 10);

    // All sequence numbers should be unique
    let mut seq_numbers: Vec<_> = events.iter().map(|e| e.sequence_nr).collect();
    seq_numbers.sort();
    assert_eq!(seq_numbers, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}

#[tokio::test]
async fn test_idempotency_across_transaction_types() {
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());

    let dedup_key = DeduplicationKey::new("deposit:1:1".to_string());

    let metadata1 = EventMetadata {
        client_id: 1,
        tx_id: 1,
        timestamp: chrono::Utc::now(),
        deduplication_key: dedup_key.clone(),
    };

    let envelope1 = journal
        .append(
            TransactionTypeEvent::Deposited(Deposited {
                client_id: 1,
                tx_id: 1,
                amount: 100.0,
            }),
            metadata1,
        )
        .await
        .unwrap();

    let metadata2 = EventMetadata {
        client_id: 1,
        tx_id: 1,
        timestamp: chrono::Utc::now(),
        deduplication_key: dedup_key.clone(),
    };

    let envelope2 = journal
        .append(
            TransactionTypeEvent::Deposited(Deposited {
                client_id: 1,
                tx_id: 1,
                amount: 100.0,
            }),
            metadata2,
        )
        .await
        .unwrap();

    assert_eq!(envelope1.sequence_nr, envelope2.sequence_nr);

    // Journal should only have one event
    let events = journal.replay(None).await.unwrap();
    assert_eq!(events.len(), 1);
}
