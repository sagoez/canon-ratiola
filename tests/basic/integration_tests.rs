use crate::context::*;

#[tokio::test]
async fn test_complex_scenario_from_spec() {
    let mut ctx = TestContext::new();

    ctx.process(deposit(1, 1, 1.0), 1).await.unwrap();
    ctx.process(deposit(1, 3, 2.0), 1).await.unwrap();
    ctx.process(withdrawal(1, 4, 1.5), 1).await.unwrap();

    ctx.assert_balances(1.5, 0.0, 1.5);
}
