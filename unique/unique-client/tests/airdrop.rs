#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mint_with_context() {
    let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
    let mut root_account = context.root_account().await;

    let account = context.gen_account();
    let create_txn = context.create_user_account_by(&mut root_account, &account);

    let mint_amount = 10_000_000;
    let mint_account_txn = root_account.sign_with_transaction_builder(
        context
            .transaction_factory()
            .mint(account.address(), mint_amount),
    );

    context
        .commit_block(&vec![create_txn.clone(), mint_account_txn.clone()])
        .await;
}
