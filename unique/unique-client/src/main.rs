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
    use std::{io::Write, path::PathBuf, str::FromStr};
    use toml::toml;

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
        let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
        let client = match &context.api_specific_config {
            ApiSpecificConfig::V1(s) => {
                Client::new(url::Url::parse(&format!("http://{s}")).unwrap())
            },
        };
        let mut root_account = context.root_account().await;
        let mut module_account = context.gen_account();

        let transfer_txn =
            context.account_transfer(&mut root_account, &module_account, 1_000_000_000);
        context.commit_block(&[transfer_txn]).await;

        let module_address_hex = module_account.address().to_hex_literal();
        let module_name = "unique".to_string();

        let toml_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
            .join(&format!("../{}/Move.toml", "toycoin"));
        let mut file = std::fs::File::create(toml_path.clone()).unwrap();
        let move_toml = toml! {
            [package]
            name = "Toycoin"
            version = "0.0.0"

            [addresses]
            std = "0x1"
            toycoin = module_address_hex

            [dependencies]
            AptosStdlib = { local = "../../aptos-move/framework/aptos-stdlib" }
        };
        file.write_all(move_toml.to_string().as_bytes()).unwrap();

        let named_addresses = vec![(module_name.clone(), module_account.address())];
        let path =
            PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join(&format!("../{}/", "toycoin"));

        let payload = TestContext::build_package(path.clone(), named_addresses);
        context.publish_package(&mut module_account, payload).await;

        client
            .get_account_module(module_account.address(), "unique")
            .await
            .unwrap();

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
