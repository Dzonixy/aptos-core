use aptos_api_test_context::{
    current_function_name, new_test_context, ApiSpecificConfig, TestContext,
};
use aptos_config::config::NodeConfig;
use aptos_sdk::rest_client::Client;
use aptos_types::transaction::{Script, TransactionArgument};
use std::path::PathBuf;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_and_publish() {
    // aptos_logger::Logger::init_for_testing();
    let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
    let client = match &context.api_specific_config {
        ApiSpecificConfig::V1(s) => Client::new(url::Url::parse(&format!("http://{s}")).unwrap()),
    };

    let mut module_account = context.create_account().await;
    let mut root_account = context.root_account().await;

    let transfer_txn = context.account_transfer(&mut root_account, &module_account, 1_000_000_000);
    context.commit_block(&[transfer_txn]).await;

    let named_address = "toycoin".to_string();
    let named_addresses = vec![(named_address.clone(), module_account.address())];
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join(&format!("../{}/", "toycoin"));

    let payload = TestContext::build_package(path.clone(), named_addresses);
    context.publish_package(&mut module_account, payload).await;

    assert_ne!(
        client
            .get_account_module(module_account.address(), "unique")
            .await
            .unwrap()
            .into_inner()
            .bytecode
            .0
            .len(),
        0
    );
    let script_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("../toycoin/build/Toycoin/bytecode_scripts/main.mv");
    let code = std::fs::read(script_path).unwrap();

    let script_txn = module_account.sign_with_transaction_builder(
        context.transaction_factory().script(Script::new(
            code,
            vec![],
            vec![TransactionArgument::U64(1), TransactionArgument::U64(1)],
        )),
    );

    context.commit_block(&[script_txn]).await;
}
