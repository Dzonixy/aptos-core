use aptos_framework::BuildOptions;
use aptos_language_e2e_tests::account::Account;
use aptos_sdk::move_types::{identifier::Identifier, language_storage::StructTag};
use aptos_types::transaction::{TransactionArgument, TransactionStatus};
use e2e_move_tests::MoveHarness;
use serde::Deserialize;
use std::path::PathBuf;

#[test]
fn deserializing_resources() {
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
        .join("../toycoin/build/Toycoin/bytecode_scripts/main.mv");
    let mut code = std::fs::read(script_path).unwrap();

    let number = 42;
    let expected_recource = UniqueResource {
        number,
        msg: "hello world".to_string(),
    };
    let mut script_txn = h.create_script(&payer, code, vec![], vec![
        TransactionArgument::U64(1),
        TransactionArgument::U64(1),
        TransactionArgument::U64(expected_recource.number),
        TransactionArgument::U8Vector(expected_recource.msg.to_owned().into_bytes()),
    ]);
    assert_eq!(
        h.run(script_txn),
        TransactionStatus::Keep(aptos_types::transaction::ExecutionStatus::Success)
    );

    let mut resource = h
        .read_resource::<UniqueResource>(payer.address(), StructTag {
            address: module_account.address().to_owned(),
            module: Identifier::new("unique").unwrap(),
            name: Identifier::new("UniqueResource").unwrap(),
            type_params: vec![],
        })
        .unwrap();

    assert_eq!(resource, expected_recource);

    script_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("../toycoin/build/Toycoin/bytecode_scripts/add_one_number.mv");
    code = std::fs::read(script_path).unwrap();

    script_txn = h.create_script(&payer, code, vec![], vec![]);
    assert_eq!(
        h.run(script_txn),
        TransactionStatus::Keep(aptos_types::transaction::ExecutionStatus::Success)
    );

    resource = h
        .read_resource::<UniqueResource>(payer.address(), StructTag {
            address: module_account.address().to_owned(),
            module: Identifier::new("unique").unwrap(),
            name: Identifier::new("UniqueResource").unwrap(),
            type_params: vec![],
        })
        .unwrap();

    assert_eq!(number + 1, resource.number);
}

#[derive(Deserialize, Debug, PartialEq, PartialOrd)]
pub struct UniqueResource {
    pub number: u64,
    pub msg: String,
}
