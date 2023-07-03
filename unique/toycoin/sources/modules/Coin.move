module toycoin::unique {
    use 0x1::aptos_account;
    use 0x1::block;
    use 0x1::coin;
 
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

    // public fun mint(value: u64): UniqueToken {
    //     UniqueToken { value }
    // }

    // public fun value(unique_token: &UniqueToken): u64 {
    //     unique_token.value
    // }

    // public fun burn(unique_token: UniqueToken): u64 {
    //     let UniqueToken { value } = unique_token;
    //     value
    // }
}
