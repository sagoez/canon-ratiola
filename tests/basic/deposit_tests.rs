use crate::context::*;

#[tokio::test]
async fn test_deposit_increases_balance() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();

    ctx.assert_balances(100.0, 0.0, 100.0);
    assert!(!ctx.is_frozen());
}

#[tokio::test]
async fn test_multiple_deposits() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 50.0), 1).await.unwrap();
    ctx.process(deposit(1, 2, 75.5), 1).await.unwrap();
    ctx.process(deposit(1, 3, 24.5), 1).await.unwrap();

    ctx.assert_balances(150.0, 0.0, 150.0);
}

#[tokio::test]
async fn test_precision_four_decimal_places() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 1.2345), 1).await.unwrap();
    ctx.process(deposit(1, 2, 2.6789), 1).await.unwrap();

    // Should handle 4 decimal places
    assert_eq!(ctx.total(), 3.9134);
}
