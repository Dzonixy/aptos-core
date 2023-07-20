use aptos_api_test_context::TestContext;
use aptos_cached_packages::aptos_stdlib;
use aptos_language_e2e_tests::{
    account::{Account, AccountData},
    executor::FakeExecutor,
};
use aptos_sdk::move_types::{identifier::Identifier, language_storage::ModuleId};
use aptos_types::{
    account_config::AccountResource,
    transaction::{ExecutionStatus, Module, Script, TransactionArgument, TransactionStatus},
};
use std::{path::PathBuf, thread};

#[test]
fn mint_to_new_account() {
    let mut executor = FakeExecutor::from_head_genesis();
    let mut root = Account::new_aptos_root();
    let (private_key, public_key) = aptos_vm_genesis::GENESIS_KEYPAIR.clone();
    root.rotate_key(private_key, public_key);

    let toycoin_account = executor.create_accounts(1, 1_000_000_000, 0).pop().unwrap();
    let unique_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("../toycoin/build/Toycoin/bytecode_modules/unique.mv");
    let named_address = "toycoin".to_string();
    let named_addresses = vec![(named_address.clone(), *toycoin_account.address())];
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join(&format!("../{}/", "toycoin"));
    TestContext::build_package(path.clone(), named_addresses);
    let toycoin_bytecode = std::fs::read(unique_path).unwrap();
    executor.add_module(
        &ModuleId::new(
            *toycoin_account.address(),
            Identifier::new("toycoin").unwrap(),
        ),
        toycoin_bytecode,
    );

    let script_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("../toycoin/build/Toycoin/bytecode_scripts/main.mv");
    let code = std::fs::read(script_path).unwrap();
    let script_txn = root
        .transaction()
        .script(Script::new(code, vec![], vec![
            TransactionArgument::U64(1),
            TransactionArgument::U64(1),
        ]))
        .gas_unit_price(100)
        .sequence_number(0)
        .sign();

    let script_output = executor.execute_transaction(script_txn);

    assert_eq!(
        script_output.status(),
        &TransactionStatus::Keep(ExecutionStatus::Success)
    );
}
