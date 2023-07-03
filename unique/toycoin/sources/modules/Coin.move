module toycoin::unique {
    // use 0x1::aptos_account;
    // use 0x1::block;
    // use 0x1::coin;
    use marketplace::coin_listing;
 
    struct Unique has drop, copy {
        value: u64,
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
