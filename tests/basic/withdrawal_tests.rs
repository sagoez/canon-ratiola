use crate::context::*;

#[tokio::test]
async fn test_withdrawal_decreases_balance() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();
    ctx.process(withdrawal(1, 2, 30.0), 1).await.unwrap();

    ctx.assert_balances(70.0, 0.0, 70.0);
}

#[tokio::test]
async fn test_withdrawal_with_insufficient_funds_fails() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 50.0), 1).await.unwrap();

    let result = ctx.process(withdrawal(1, 2, 100.0), 1).await;

    assert!(
        result.is_err(),
        "Withdrawal should fail with insufficient funds"
    );
    ctx.assert_balances(50.0, 0.0, 50.0);
}

#[tokio::test]
async fn test_withdrawal_with_exact_balance() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();
    ctx.process(withdrawal(1, 2, 100.0), 1).await.unwrap();

    ctx.assert_balances(0.0, 0.0, 0.0);
}
