module toycoin::unique {
    // use 0x1::aptos_account;
    // use 0x1::block;
    // use 0x1::coin;
    // use marketplace::coin_listing;
    use std::event::{Self, EventHandle};
    use std::signer;
    use std::vector;

    struct UniqueEvent has drop, store { msg: vector<u8> }

    struct UniqueEventHandle has key, store {
        // unique_events: EventHandle<UniqueEvent>,
        // unique_events: UniqueEvent,
        number: u64,
        msg: vector<u8>,
    }
 
    struct Unique has drop, copy {
        value: u64,

    }

    public fun new_unique(account: &signer, number: u64, msg: vector<u8>) {
        move_to<UniqueEventHandle>(account, 
        UniqueEventHandle {
            number,
            msg,
        });
    }

    public fun sum(a: u64, b: u64): u64 {
        a + b
    }

    public fun new(): Unique {
        Unique {
            value: 1,
        }
    }

    public fun gimme_five(): u64 {
        5
    }
}


// event::emit_event<DepositEvent>(
//             &mut coin_store.deposit_events,
//             DepositEvent { amount: coin.value },
//         );