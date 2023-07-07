use std::{path::PathBuf, vec};

use aptos_framework::BuildOptions;
use aptos_language_e2e_tests::account::Account;
use aptos_sdk::move_types::{identifier::Identifier, language_storage::StructTag};
use aptos_types::transaction::{TransactionArgument, TransactionStatus};
use e2e_move_tests::MoveHarness;
use serde::{Deserialize, Serialize};

#[test]
fn parse_struct() {
    let mut h = MoveHarness::new();

    let _root = Account::new_aptos_root();
    let (_private_key, _public_key) = aptos_vm_genesis::GENESIS_KEYPAIR.clone();

    let module_account = h.new_account_with_key_pair();
    let payer = h.new_account_with_key_pair();

    let mut build_options = BuildOptions::default();
    build_options
        .named_addresses
        .insert("toycoin".to_string(), module_account.address().clone());
    let package_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("../toycoin");
    h.publish_package_with_options(&module_account, &package_path, build_options);

    let mut script_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("../toycoin/build/Toycoin/bytecode_scripts/parse_struct_from_vec.mv");
    let code = std::fs::read(script_path).unwrap();

    let struct_to_parse = ParsedStruct {
        number_u64: 23299423,
        number_u8: 3,
    };

    let mut bytes: Vec<u8> = vec![];
    bcs::serialize_into(&mut bytes, &struct_to_parse).unwrap();

    let mut script_txn = h.create_script(
        &payer,
        code,
        vec![],
        vec![TransactionArgument::U8Vector(bytes)],
    );
    assert_eq!(
        h.run(script_txn),
        TransactionStatus::Keep(aptos_types::transaction::ExecutionStatus::Success)
    );
}

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Serialize)]
struct ParsedStruct {
    number_u64: u64,
    number_u8: u8,
}
