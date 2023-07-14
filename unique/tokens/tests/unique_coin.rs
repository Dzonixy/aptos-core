use std::{path::PathBuf, vec};

use aptos_framework::BuildOptions;
use aptos_language_e2e_tests::account::Account;
use aptos_sdk::move_types::{
    language_storage::{StructTag, TypeTag},
    transaction_argument::convert_txn_args,
};
use aptos_types::{
    account_config::{addresses, CoinStoreResource},
    transaction::{TransactionArgument, TransactionStatus},
};
use e2e_move_tests::MoveHarness;
use move_core_types::ident_str;

#[test]
fn unique_coin() {
    let mut h = MoveHarness::new();

    let _root = Account::new_aptos_root();
    let (_private_key, _public_key) = aptos_vm_genesis::GENESIS_KEYPAIR.clone();

    let module_account = h.new_account_with_key_pair();
    let alice = h.new_account_with_key_pair();

    let mut build_options = BuildOptions::default();
    build_options
        .named_addresses
        .insert("tokens".to_string(), module_account.address().clone());
    let package_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("../tokens");
    h.publish_package_with_options(&module_account, &package_path, build_options);

    let mut name_a_bytes = vec![];
    bcs::serialize_into(&mut name_a_bytes, "CoinA").unwrap();
    let mut symbol_a_bytes = vec![];
    bcs::serialize_into(&mut symbol_a_bytes, "CoinA").unwrap();

    let unique_coin_struct_tag = StructTag {
        address: *module_account.address(),
        module: ident_str!("unique_coin").to_owned(),
        name: ident_str!("CoinA").to_owned(),
        type_params: vec![],
    };
    let unique_coin_a_type = TypeTag::Struct(Box::new(unique_coin_struct_tag.clone()));

    assert_eq!(
        h.run_entry_function(
            &module_account,
            str::parse(&format!(
                "{}::unique_coin::initialize_coin",
                module_account.address()
            ))
            .unwrap(),
            vec![unique_coin_a_type],
            convert_txn_args(&[
                TransactionArgument::U8Vector(name_a_bytes),
                TransactionArgument::U8Vector(symbol_a_bytes)
            ]),
        ),
        TransactionStatus::Keep(aptos_types::transaction::ExecutionStatus::Success)
    );

    assert_eq!(
        h.run_entry_function(
            &alice,
            str::parse(&format!(
                "{}::unique_coin::register_unique_coin",
                module_account.address()
            ))
            .unwrap(),
            vec![],
            convert_txn_args(&[]),
        ),
        TransactionStatus::Keep(aptos_types::transaction::ExecutionStatus::Success)
    );

    let unique_coin_struct_tag = StructTag {
        address: *module_account.address(),
        module: ident_str!("unique_coin").to_owned(),
        name: ident_str!("CoinA").to_owned(),
        type_params: vec![],
    };
    let unique_coin_type = TypeTag::Struct(Box::new(unique_coin_struct_tag.clone()));
    let transfer_amount = 50000;

    assert_eq!(
        h.run_entry_function(
            &module_account,
            str::parse(&format!("0x1::coin::transfer")).unwrap(),
            vec![unique_coin_type.clone()],
            convert_txn_args(&[
                TransactionArgument::Address(*alice.address()),
                TransactionArgument::U64(transfer_amount),
            ]),
        ),
        TransactionStatus::Keep(aptos_types::transaction::ExecutionStatus::Success)
    );
    let payer_unique_coin_store = h
        .read_resource::<CoinStoreResource>(
            alice.address(),
            StructTag {
                address: addresses::CORE_CODE_ADDRESS,
                module: ident_str!("coin").to_owned(),
                name: ident_str!("CoinStore").to_owned(),
                type_params: vec![unique_coin_type.clone()],
            },
        )
        .unwrap();

    assert_eq!(payer_unique_coin_store.coin(), transfer_amount);
}
