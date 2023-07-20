use aptos_api_test_context::{
    current_function_name, new_test_context, ApiSpecificConfig, TestContext,
};
use aptos_config::config::NodeConfig;
use aptos_sdk::rest_client::Client;
use std::path::PathBuf;
// use sealed_test::prelude::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn publish_marketplace() {
    let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
    let client = match &context.api_specific_config {
        ApiSpecificConfig::V1(s) => Client::new(url::Url::parse(&format!("http://{s}")).unwrap()),
    };
    let mut root_account = context.root_account().await;

    let named_addresses = vec![("marketplace".to_string(), root_account.address())];
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("../marketplace");

    let payload = TestContext::build_package(path, named_addresses);
    context.publish_package(&mut root_account, payload).await;

    let _module = client
        .get_account_module(root_account.address(), "collection_offer")
        .await
        .unwrap();

    // TODO: test marketplace
}
