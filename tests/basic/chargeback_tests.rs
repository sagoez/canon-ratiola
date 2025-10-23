use crate::context::*;

#[tokio::test]
async fn test_chargeback_reverses_transaction_and_freezes_account() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();
    ctx.process(dispute(1, 1), 1).await.unwrap();
    ctx.assert_balances(0.0, 100.0, 100.0);

    ctx.process(chargeback(1, 1), 1).await.unwrap();

    ctx.assert_balances(0.0, 0.0, 0.0);
    assert!(ctx.is_frozen(), "Account should be frozen after chargeback");
}

#[tokio::test]
async fn test_chargeback_without_dispute_ignored() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();

    // Try chargeback without dispute - spec says "ignore"
    // Implementation: event handler returns None (defense in depth)
    let result = ctx.process(chargeback(1, 1), 1).await;
    assert!(result.is_err(), "Chargeback without dispute should fail");

    ctx.assert_balances(100.0, 0.0, 100.0);
    assert!(!ctx.is_frozen());
}
