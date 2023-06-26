#[tokio::main]
async fn main() {}

#[cfg(test)]
mod tests {
    use aptos_api_test_context::{
        current_function_name, new_test_context, ApiSpecificConfig, TestContext,
    };
    use aptos_config::config::NodeConfig;
    use aptos_sdk::{
        rest_client::{aptos_api_types::Address, Client},
        types::transaction::{Script, TransactionArgument},
    };
    use serde_json::json;

    use aptos_sdk::types::account_address::AccountAddress;
    use move_binary_format::CompiledModule;
    use sealed_test::prelude::*;
    use std::{path::PathBuf, str::FromStr};

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

    // #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn build_and_publish() {
        // aptos_logger::Logger::new().init();

        let move_toml_string = std::fs::read_to_string(
            PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
                .join(&format!("../{}/Move.toml", "toycoin")),
        )
        .unwrap();

        let _move_toml = &move_toml_string.parse::<toml::Value>().unwrap();

        let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
        let client = match &context.api_specific_config {
            ApiSpecificConfig::V1(s) => {
                Client::new(url::Url::parse(&format!("http://{s}")).unwrap())
            },
        };
        let mut root_account = context.root_account().await;

        let module_name = "unique".to_string();
        let module_address = AccountAddress::from_str("0x2").unwrap();

        let named_addresses = vec![(module_name.clone(), root_account.address())];
        let path =
            PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join(&format!("../{}/", "toycoin"));

        context
            .publish_package(
                &mut root_account,
                TestContext::build_package(path.clone(), named_addresses),
            )
            .await;

        let module = client
            .get_account_module(root_account.address(), "unique")
            .await
            .unwrap();

        println!("{module:?}");

        // println!(
        //     "{:?}",
        //     client
        //         .get_account_module(AccountAddress::from_str("0x1").unwrap(), "signer")
        //         .await
        //         .unwrap()
        // );

        // let module_bytecode = client
        //     .get_account_module(module_address, &module_name)
        //     .await
        //     .unwrap();

        // let compiled_script = std::fs::read(
        //     PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        //         .join("../toycoin/build/toycoin/bytecode_scripts/main.mv"),
        // )
        // .unwrap();

        // let script_txn =
        //     root_account.sign_with_transaction_builder(context.transaction_factory().script(
        //         Script::new(compiled_script, vec![], vec![TransactionArgument::U64(1)]),
        //     ));

        // context.commit_block(&vec![script_txn]).await;
    }

    #[sealed_test(env = [("RUST_MIN_STACK", "10485760")])]
    fn publish_toycoin_test() {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap()
            .block_on(build_and_publish())
    }

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

        let module = client.get_account_modules(module_address).await.unwrap();

        // TODO: test marketplace
    }

    #[sealed_test(env = [("RUST_MIN_STACK", "10485760")])]
    fn publish_marketplace_test() {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap()
            .block_on(publish_marketplace())
    }

    #[tokio::test]
    async fn compile_script_from_string() {
        let _script_string = std::fs::read_to_string(
            PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
                .join("../toycoin/sources/scripts/test-coin.move"),
        )
        .unwrap();
    }
}
