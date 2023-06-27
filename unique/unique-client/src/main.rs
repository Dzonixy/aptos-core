#[tokio::main]
async fn main() {
    // AptosNodeArgs::parse().run()
}

#[cfg(test)]
mod tests {
    use aptos_api_test_context::{
        current_function_name, new_test_context, ApiSpecificConfig, TestContext,
    };
    use aptos_config::config::NodeConfig;
    use aptos_sdk::{
        rest_client::Client,
        types::transaction::{Script, TransactionArgument},
    };

    use aptos_sdk::types::account_address::AccountAddress;
    // use sealed_test::prelude::*;
    use std::{path::PathBuf, str::FromStr};

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn client_build() {
        let _client = Client::new(url::Url::parse("http://0.0.0.0:8080").unwrap());
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn build_and_publish() {
        // aptos_logger::Logger::new().init();
        let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
        let client = match &context.api_specific_config {
            ApiSpecificConfig::V1(s) => {
                Client::new(url::Url::parse(&format!("http://{s}")).unwrap())
            },
        };

        let mut module_account = context.create_account().await;
        let mut root_account = context.root_account().await;

        let transfer_txn =
            context.account_transfer(&mut root_account, &module_account, 1_000_000_000);
        context.commit_block(&[transfer_txn]).await;

        let named_address = "toycoin".to_string();
        let named_addresses = vec![(named_address.clone(), module_account.address())];
        let path =
            PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join(&format!("../{}/", "toycoin"));

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

        let compiled_script = std::fs::read(
            PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
                .join("../toycoin/build/toycoin/bytecode_scripts/main.mv"),
        )
        .unwrap();
        let script_txn =
            root_account.sign_with_transaction_builder(context.transaction_factory().script(
                Script::new(compiled_script, vec![], vec![TransactionArgument::U64(1)]),
            ));
        context.commit_block(&vec![script_txn]).await;
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn publish_marketplace() {
        let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
        let client = match &context.api_specific_config {
            ApiSpecificConfig::V1(s) => {
                Client::new(url::Url::parse(&format!("http://{s}")).unwrap())
            },
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

    // #[sealed_test(env = [("RUST_MIN_STACK", "10485760")])]
    // fn publish_marketplace_test() {
    //     tokio::runtime::Builder::new_multi_thread()
    //         .worker_threads(1)
    //         .enable_all()
    //         .build()
    //         .unwrap()
    //         .block_on(publish_marketplace())
    // }

    #[tokio::test]
    async fn compile_script_from_string() {
        let _script_string = std::fs::read_to_string(
            PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
                .join("../toycoin/sources/scripts/test-coin.move"),
        )
        .unwrap();
    }
}
