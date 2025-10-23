use payment::adapter::InMemoryJournal;
use payment::domain::*;
use payment::port::Journal;
use std::sync::Arc;

#[tokio::test]
async fn test_events_applied_in_sequence_order() {
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());

    let metadata1 = EventMetadata {
        client_id: 1,
        tx_id: 1,
        timestamp: chrono::Utc::now(),
        deduplication_key: DeduplicationKey::new("deposit:1:1".to_string()),
    };

    let metadata2 = EventMetadata {
        client_id: 1,
        tx_id: 2,
        timestamp: chrono::Utc::now(),
        deduplication_key: DeduplicationKey::new("deposit:1:2".to_string()),
    };

    let metadata3 = EventMetadata {
        client_id: 1,
        tx_id: 3,
        timestamp: chrono::Utc::now(),
        deduplication_key: DeduplicationKey::new("withdraw:1:3".to_string()),
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

    let envelope2 = journal
        .append(
            TransactionTypeEvent::Deposited(Deposited {
                client_id: 1,
                tx_id: 2,
                amount: 50.0,
            }),
            metadata2,
        )
        .await
        .unwrap();

    let envelope3 = journal
        .append(
            TransactionTypeEvent::Withdrawn(Withdrawn {
                client_id: 1,
                tx_id: 3,
                amount: 30.0,
            }),
            metadata3,
        )
        .await
        .unwrap();

    assert_eq!(envelope1.sequence_nr, 1);
    assert_eq!(envelope2.sequence_nr, 2);
    assert_eq!(envelope3.sequence_nr, 3);

    let events = journal.replay(None).await.unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].sequence_nr, 1);
    assert_eq!(events[1].sequence_nr, 2);
    assert_eq!(events[2].sequence_nr, 3);
}

#[tokio::test]
async fn test_replay_from_specific_sequence() {
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());

    for i in 1..=5 {
        let metadata = EventMetadata {
            client_id: 1,
            tx_id: i,
            timestamp: chrono::Utc::now(),
            deduplication_key: DeduplicationKey::new(format!("deposit:1:{}", i)),
        };

        journal
            .append(
                TransactionTypeEvent::Deposited(Deposited {
                    client_id: 1,
                    tx_id: i,
                    amount: 10.0,
                }),
                metadata,
            )
            .await
            .unwrap();
    }

    let events = journal.replay(Some(3)).await.unwrap();

    // Should return sequences 3, 4, and 5
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].sequence_nr, 3);
    assert_eq!(events[1].sequence_nr, 4);
    assert_eq!(events[2].sequence_nr, 5);
}

#[tokio::test]
async fn test_highest_sequence_tracking() {
    let journal: Arc<dyn Journal + Send + Sync> = Arc::new(InMemoryJournal::new());

    // Initially should be None
    let highest = journal.highest_sequence().await.unwrap();
    assert_eq!(highest, None);

    // After one event
    let metadata = EventMetadata {
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
            metadata,
        )
        .await
        .unwrap();

    let highest = journal.highest_sequence().await.unwrap();
    assert_eq!(highest, Some(1));

    for i in 2..=10 {
        let metadata = EventMetadata {
            client_id: 1,
            tx_id: i,
            timestamp: chrono::Utc::now(),
            deduplication_key: DeduplicationKey::new(format!("deposit:1:{}", i)),
        };

        journal
            .append(
                TransactionTypeEvent::Deposited(Deposited {
                    client_id: 1,
                    tx_id: i,
                    amount: 10.0,
                }),
                metadata,
            )
            .await
            .unwrap();
    }

    let highest = journal.highest_sequence().await.unwrap();
    assert_eq!(highest, Some(10));
}
