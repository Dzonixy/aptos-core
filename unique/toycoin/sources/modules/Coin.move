module toycoin::unique {
    struct Unique has key {
        value: u64,
    }

    public fun gimme_five(): u8 {
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
