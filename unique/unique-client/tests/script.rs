use std::path::PathBuf;

use aptos_api_test_context::TestContext;
use aptos_cached_packages::aptos_stdlib;
use aptos_language_e2e_tests::{account::Account, executor::FakeExecutor};
use aptos_types::transaction::{ExecutionStatus, Script, TransactionArgument, TransactionStatus};

#[test]
fn mint_to_new_account() {
    let mut executor = FakeExecutor::from_head_genesis();
    let mut root = Account::new_aptos_root();
    let (private_key, public_key) = aptos_vm_genesis::GENESIS_KEYPAIR.clone();
    root.rotate_key(private_key, public_key);

    // Create and publish a sender with TXN_RESERVED coins, also note how
    // many were there before.
    let new_account = executor.create_raw_account_data(0, 0);
    executor.add_account_data(&new_account);
    let supply_before = executor.read_coin_supply().unwrap();

    let mint_amount = 1_000_000;
    let txn = root
        .transaction()
        .payload(aptos_stdlib::aptos_coin_mint(
            *new_account.address(),
            mint_amount,
        ))
        .gas_unit_price(100)
        .sequence_number(0)
        .sign();

    // This generates output (WriteSet), but it needs to be written as "state change" in DB.
    let output = executor.execute_transaction(txn);

    // This is where it is written in StateView
    // You get WriteSet from TransactionOutput and then you apply it.
    executor.apply_write_set(output.write_set());
    // Check that supply changed.
    let supply_after = executor.read_coin_supply().unwrap();
    assert_eq!(
        supply_after,
        supply_before + (mint_amount as u128) - (output.gas_used() * 100) as u128
    );

    // checks that the airdrop succeded
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(ExecutionStatus::Success),
    );

    let named_addresses = vec![("toycoin".to_string(), *root.address())];
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("../toycoin");

    let payload = TestContext::build_package(path, named_addresses);

    let publish_txn = root
        .transaction()
        .payload(payload)
        .gas_unit_price(100)
        .sequence_number(1)
        .sign();

    let publish_output = executor.execute_transaction(publish_txn);
    executor.apply_write_set(publish_output.write_set());

    assert_eq!(
        publish_output.status(),
        &TransactionStatus::Keep(ExecutionStatus::Success)
    );

    let script_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("../toycoin/build/Toycoin/bytecode_scripts/main.mv");
    let code = std::fs::read(script_path).unwrap();

    let script_txn = root
        .transaction()
        .script(Script::new(
            code,
            vec![],
            vec![TransactionArgument::U64(1), TransactionArgument::U64(1)],
        ))
        .gas_unit_price(100)
        .sequence_number(2)
        .sign();

    let script_output = executor.execute_transaction(script_txn);

    assert_eq!(
        script_output.status(),
        &TransactionStatus::Keep(ExecutionStatus::Success)
    );
}
