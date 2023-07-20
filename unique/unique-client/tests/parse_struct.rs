use aptos_framework::{natives::any::Any, BuildOptions};
use aptos_language_e2e_tests::account::Account;
use aptos_sdk::move_types::{
    identifier::Identifier, language_storage::StructTag, transaction_argument::convert_txn_args,
};
use aptos_types::transaction::{TransactionArgument, TransactionStatus};
use e2e_move_tests::MoveHarness;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, vec};

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

    let script_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("../toycoin/build/Toycoin/bytecode_scripts/parse_struct_from_vec.mv");
    let code = std::fs::read(script_path).unwrap();

    let ps = ParsedStruct {
        number_u64: 1024,
        number_u8: 8,
    };
    let mut bytes: Vec<u8> = vec![];
    bcs::serialize_into(&mut bytes, &ps).unwrap();
    let struct_to_parse = UniqueResource {
        number: 23299423,
        msg: 3,
        unique_data: Any {
            data: bytes.clone(),
            type_name: stringify!(ParsedStruct).to_string(),
        },
    };

    assert_eq!(
        h.run_entry_function(
            &payer,
            str::parse(&format!("{}::unique::new_unique", module_account.address())).unwrap(),
            vec![],
            convert_txn_args(&[
                TransactionArgument::U64(struct_to_parse.number),
                TransactionArgument::U8Vector(b"Hello Unique!".to_vec()),
            ]),
        ),
        TransactionStatus::Keep(aptos_types::transaction::ExecutionStatus::Success)
    );

    let script_txn = h.create_script(&payer, code, vec![], vec![]);
    assert_eq!(
        h.run(script_txn),
        TransactionStatus::Keep(aptos_types::transaction::ExecutionStatus::Success)
    );
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
struct UniqueResource {
    number: u64,
    msg: u8,
    unique_data: Any,
}

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Serialize)]
struct ParsedStruct {
    number_u64: u64,
    number_u8: u8,
}
