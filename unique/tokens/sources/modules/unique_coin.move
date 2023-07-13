module tokens::unique_coin {
    use aptos_framework::coin::{Self, BurnCapability, FreezeCapability, MintCapability};
    use std::string; 
    use std::signer;

    struct UniqueCoin has key {}

    struct UniqueCoinManagement has key {
        burn_capability: BurnCapability<UniqueCoin>,
        freeze_capability: FreezeCapability<UniqueCoin>,
        mint_capability: MintCapability<UniqueCoin>,
    }

    public entry fun initialize_unique_coin(account: &signer) {
        // Register the coin (Create CoinStore<UniqueCoin> resource)
        coin::register<UniqueCoin>(account);
        // Initialize the coin (Create CoinInfo<UniqueCoin> and get 
        // burn, freeze and mint capability's handles to store)
        let (burn_capability, freeze_capability, mint_capability) = 
            coin::initialize<UniqueCoin>(
                account,
                string::utf8(b"Unique Coin"),
                string::utf8(b"UC"),
                9,
                false,
            );
        // Mint the coin
        let unique_coin = coin::mint<UniqueCoin>(
            18446744073709551615,
            &mint_capability,
        );
        // Once the coin is minted, deposit it in CoinStore resource
        coin::deposit<UniqueCoin>(signer::address_of(account), unique_coin);
        move_to(
            account, 
            UniqueCoinManagement {
                burn_capability,
                freeze_capability,
                mint_capability,
            }
        );
    }

    public entry fun register_unique_coin(account: &signer) {
        coin::register<UniqueCoin>(account);
    }

}