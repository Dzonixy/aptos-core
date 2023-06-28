use aptos_framework::BuiltPackage;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join(&format!("../{}/", "toycoin"));
    BuiltPackage::build(path, aptos_framework::BuildOptions::default()).unwrap();
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

    #[tokio::test]
    async fn compile_script_from_string() {
        let _script_string = std::fs::read_to_string(
            PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
                .join("../toycoin/sources/scripts/test-coin.move"),
        )
        .unwrap();
    }
}
