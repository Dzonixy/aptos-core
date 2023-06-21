#[tokio::main]
async fn main() {}

#[cfg(test)]
mod tests {
    use aptos_api_test_context::{current_function_name, new_test_context, TestContext};
    use aptos_config::config::NodeConfig;
    use aptos_sdk::{rest_client::Client, types::transaction::Script};

    use move_binary_format::CompiledModule;
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 6)]
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
        println!("module binary: {:?}", binary);

        let code = package.extract_script_code()[0].clone();
        println!("code: {:?}", code);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 6)]
    async fn build_and_publish() {
        let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
        let mut root_account = context.root_account().await;

        let package_account = context.gen_account();
        let txn = context.create_user_account_by(&mut root_account, &package_account);
        context.commit_block(&vec![txn]).await;

        let named_addresses = vec![("toycoin".to_string(), package_account.address())];
        let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("../toycoin/");

        let payload = TestContext::build_package(path.clone(), named_addresses);
        println!(
            "Transaction Payload: {:?}",
            payload.clone().into_entry_function()
        );

        context.publish_package(&mut root_account, payload).await;

        root_account.sign_with_transaction_builder(
            context
                .transaction_factory()
                .script(Script::new(vec![], vec![], vec![])),
        );
    }

    async fn publish_marketplace() {
        let mut context = new_test_context(current_function_name!(), NodeConfig::default(), false);
        let mut root_account = context.root_account().await;

        let package_account = context.gen_account();
        let txn = context.create_user_account_by(&mut root_account, &package_account);
        context.commit_block(&vec![txn]).await;

        let _named_addresses = vec![("Marketplace".to_string(), package_account.address())];
        let _path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("../marketplace");
        // let payload = TestContext::build_package(path, named_addresses);

        // context.publish_package(&mut root_account, payload).await;
    }

    #[test]
    fn publish_marketplace_test() {
        tokio::runtime::Builder::new_multi_thread()
            .thread_stack_size(104857600)
            .worker_threads(6)
            .enable_all()
            .build()
            .unwrap()
            .block_on(publish_marketplace())
    }
}
