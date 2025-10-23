use crate::context::*;

#[tokio::test]
async fn test_frozen_account_cannot_withdraw() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();
    ctx.process(deposit(1, 2, 50.0), 1).await.unwrap();
    ctx.process(dispute(1, 1), 1).await.unwrap();
    ctx.process(chargeback(1, 1), 1).await.unwrap();

    assert!(ctx.is_frozen());
    ctx.assert_balances(50.0, 0.0, 50.0);

    let result = ctx.process(withdrawal(1, 3, 10.0), 1).await;

    assert!(result.is_err(), "Withdrawal should fail on frozen account");
    ctx.assert_balances(50.0, 0.0, 50.0);
}

#[tokio::test]
async fn test_frozen_account_can_receive_deposits() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();
    ctx.process(dispute(1, 1), 1).await.unwrap();
    ctx.process(chargeback(1, 1), 1).await.unwrap();

    assert!(ctx.is_frozen());
    ctx.assert_balances(0.0, 0.0, 0.0);

    ctx.process(deposit(1, 2, 50.0), 1).await.unwrap();

    ctx.assert_balances(50.0, 0.0, 50.0);
    assert!(ctx.is_frozen());
}

#[tokio::test]
async fn test_frozen_account_can_be_disputed() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 100.0), 1).await.unwrap();
    ctx.process(deposit(1, 2, 50.0), 1).await.unwrap();
    ctx.process(dispute(1, 1), 1).await.unwrap();
    ctx.process(chargeback(1, 1), 1).await.unwrap();

    assert!(ctx.is_frozen());
    ctx.assert_balances(50.0, 0.0, 50.0);

    ctx.process(dispute(1, 2), 1).await.unwrap();

    ctx.assert_balances(0.0, 50.0, 50.0);
}
