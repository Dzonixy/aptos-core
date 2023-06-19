use std::path::PathBuf;

use aptos_api_test_context::{current_function_name, new_test_context, TestContext};
use aptos_config::config::NodeConfig;
use aptos_sdk::{rest_client::Client, types::account_address::AccountAddress};

#[tokio::main]
async fn main() {}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn account_balance() {
    // Requires node to be active
    let client = Client::new(url::Url::parse("http://0.0.0.0:8080").unwrap());
    let account_address = AccountAddress::from_hex_literal(
        "0xc01949220e66521866d86fa324d765c76a4b61f58672ea2d68ea0e9b49a10e08",
    )
    .unwrap();
    let account = client.get_account_balance(account_address).await.unwrap();

    println!("account balance: {:#?}", account);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn client_build() {
    // Requires node to be active
    let client = Client::new(url::Url::parse("http://0.0.0.0:8080").unwrap());
    println!("Client: {:#?}", client);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mint_with_context() {
    let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
    let mut root_account = context.root_account().await;

    let account = context.gen_account();
    let create_txn = context.create_user_account_by(&mut root_account, &account);
    let mint_amount = 1_000_000_000;
    let mint_txn = root_account.sign_with_transaction_builder(
        context
            .transaction_factory()
            .mint(account.address(), mint_amount),
    );

    let named_addresses = vec![("toycoin".to_string(), root_account.address())];
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("../toycoin/");
    let payload = TestContext::build_package(path, named_addresses);
    let publish_txn = context.publish_package(&mut root_account, payload).await;

    context
        .commit_block(&vec![
            create_txn.clone(),
            mint_txn.clone(),
            publish_txn.clone(),
        ])
        .await;
}
