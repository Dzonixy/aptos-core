#[tokio::main]
async fn main() {}

#[cfg(test)]
mod tests {
    use aptos_api_test_context::{
        current_function_name, new_test_context, ApiSpecificConfig, TestContext,
    };
    use aptos_config::config::NodeConfig;
    use aptos_sdk::{
        crypto::hash::TestOnlyHash,
        rest_client::Client,
        types::transaction::{Script, TransactionArgument},
    };

    use move_binary_format::CompiledModule;
    use sealed_test::prelude::*;
    use std::path::PathBuf;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn client_build() {
        let _client = Client::new(url::Url::parse("http://0.0.0.0:8080").unwrap());
        //     println!("Client: {:#?}", client);
    }

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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn build_package() {
        let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("../toycoin/");
        let build_options = aptos_framework::BuildOptions {
            with_srcs: false,
            with_abis: false,
            with_source_maps: false,
            with_error_map: false,
            ..aptos_framework::BuildOptions::default()
        };
        let package = aptos_framework::BuiltPackage::build(path, build_options)
            .expect("building package must succeed");

        let mut binary = vec![];
        let module = package.modules().collect::<Vec<&CompiledModule>>()[0];
        module.serialize(&mut binary).unwrap();

        let _code = package.extract_script_code()[0].clone();
        // println!("code: {:?}", code);
        // println!("code len : {:?}", code.len());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn build_and_publish() {
        let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
        let client = match &context.api_specific_config {
            ApiSpecificConfig::V1(s) => {
                Client::new(url::Url::parse(&format!("http://{s}")).unwrap())
            },
        };
        let mut root_account = context.root_account().await;
        let user_account = context.gen_account();

        let create_user_txn = context.create_user_account_by(&mut root_account, &user_account);
        let account_transfer_txn =
            context.account_transfer(&mut root_account, &user_account, 10_000_000_000);

        context
            .commit_block(&[create_user_txn, account_transfer_txn])
            .await;

        let named_addresses = vec![("UniqueToken".to_string(), user_account.address())];
        let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("../toycoin/");

        let payload = TestContext::build_package(path.clone(), named_addresses);
        let payload_clone = payload.clone();

        let compiled_script = std::fs::read(
            PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
                .join("../toycoin/build/toycoin/bytecode_scripts/main.mv"),
        )
        .unwrap();

        let publish_txn = context.publish_package(&mut root_account, payload).await;

        let _ = context
            // .expect_status_code(200)
            .get(&account_resources(&user_account.address().to_string()))
            .await;
        let resp = context
            // .expect_status_code(200)
            .get(&account_modules(&root_account.address().to_string()))
            .await;

        // println!("{resp:#?}");
        // context.check_golden_output(resp);
        // return;
        // let publish_txn_hash = publish_txn.test_only_hash();
        // context.commit_block(&[publish_txn]).await;

        // println!("{publish_txn_hash:?}");

        // let temp = client
        //     .get_account_modules(package_account.address())
        //     .await
        //     .unwrap();
        // println!("{temp:?}");
        // context
        //     .commit_block(&vec![root_account.sign_with_transaction_builder(
        //         context.transaction_factory().script(Script::new(
        //             compiled_script,
        //             vec![],
        //             vec![TransactionArgument::U64(1)],
        //         )),
        //     )])
        //     .await;
    }

    #[tokio::test]
    async fn compile_script_from_string() {
        let script_string = std::fs::read_to_string(
            PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
                .join("../toycoin/sources/scripts/test-coin.move"),
        )
        .unwrap();
    }

    async fn publish_marketplace() {
        let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
        let mut root_account = context.root_account().await;

        let package_account = context.gen_account();
        let txn = context.create_user_account_by(&mut root_account, &package_account);
        context.commit_block(&vec![txn]).await;

        let named_addresses = vec![("Marketplace".to_string(), package_account.address())];
        let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("../marketplace");

        let payload = TestContext::build_package(path, named_addresses);
        context.publish_package(&mut root_account, payload).await;

        // TODO: test marketplace
    }

    #[sealed_test(env = [("RUST_MIN_STACK", "10485760")])]
    fn publish_marketplace_test() {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
            .block_on(publish_marketplace())
    }

    fn account_resources(address: &str) -> String {
        format!("/accounts/{}/resources", address)
    }

    fn account_modules(address: &str) -> String {
        format!("/accounts/{}/modules", address)
    }
}
