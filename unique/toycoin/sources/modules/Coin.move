address 0x2 {

    module UniqueToken {

        struct UniqueToken has drop {
            value: u64,
        }

        public fun mint(value: u64): UniqueToken {
            UniqueToken { value }
        }

        public fun value(unique_token: &UniqueToken): u64 {
            unique_token.value
        }

        public fun burn(unique_token: UniqueToken): u64 {
            let UniqueToken { value } = unique_token;
            value
        }
    }
}