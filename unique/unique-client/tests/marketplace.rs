#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn publish_marketplace() {
    let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
    let _client = match &context.api_specific_config {
        ApiSpecificConfig::V1(s) => Client::new(url::Url::parse(&format!("http://{s}")).unwrap()),
    };
    let mut root_account = context.root_account().await;

    let module_address = AccountAddress::from_str("42").unwrap();
    let named_addresses = vec![("marketplace".to_string(), module_address)];
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("../marketplace");

    let payload = TestContext::build_package(path, named_addresses);
    context.publish_package(&mut root_account, payload).await;

    // let module = client.get_account_modules(module_address).await.unwrap();

    // TODO: test marketplace
}

// #[sealed_test(env = [("RUST_MIN_STACK", "1048576000")])]
// fn publish_toycoin_test() {
//     tokio::runtime::Builder::new_multi_thread()
//         .worker_threads(2)
//         .enable_all()
//         .build()
//         .unwrap()
//         .block_on(build_and_publish())
// }
