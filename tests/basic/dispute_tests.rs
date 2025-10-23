use crate::context::*;

#[tokio::test]
async fn test_dispute_holds_funds() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();

    ctx.process(dispute(1, 1), 1).await.unwrap();

    ctx.assert_balances(0.0, 100.0, 100.0);
    assert!(!ctx.is_frozen());
}

#[tokio::test]
async fn test_dispute_nonexistent_transaction_ignored() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();

    let result = ctx.process(dispute(1, 999), 1).await;

    assert!(result.is_err(), "Disputing nonexistent tx should fail");
    ctx.assert_balances(100.0, 0.0, 100.0);
}

#[tokio::test]
async fn test_resolve_releases_held_funds() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();
    ctx.process(dispute(1, 1), 1).await.unwrap();
    ctx.assert_balances(0.0, 100.0, 100.0);

    ctx.process(resolve(1, 1), 1).await.unwrap();

    ctx.assert_balances(100.0, 0.0, 100.0);
    assert!(!ctx.is_frozen());
}

#[tokio::test]
async fn test_resolve_without_dispute_ignored() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();

    let result = ctx.process(resolve(1, 1), 1).await;
    assert!(
        result.is_err(),
        "Resolve without dispute should fail validation"
    );

    ctx.assert_balances(100.0, 0.0, 100.0);
}
