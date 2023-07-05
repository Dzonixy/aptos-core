use std::path::PathBuf;

use aptos_framework::BuildOptions;
use aptos_language_e2e_tests::account::Account;
use aptos_types::transaction::TransactionArgument;
use e2e_move_tests::MoveHarness;

#[test]
fn implementing_test_harness() {
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
        .join("../toycoin/build/Toycoin/bytecode_scripts/main.mv");
    let code = std::fs::read(script_path).unwrap();

    let script_txn = h.create_script(
        &payer,
        code,
        vec![],
        vec![TransactionArgument::U64(1), TransactionArgument::U64(1)],
    );

    h.run(script_txn);
}
