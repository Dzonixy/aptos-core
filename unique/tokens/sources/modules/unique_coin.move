module tokens::unique_coin {
    use aptos_framework::coin::{Self, BurnCapability, FreezeCapability, MintCapability};
    use std::string; 
    use std::signer;

    struct CoinA has key {}

    struct CoinB has key {}

    struct CoinManagement<phantom C: key> has key {
        burn_capability: BurnCapability<C>,
        freeze_capability: FreezeCapability<C>,
        mint_capability: MintCapability<C>,
    }

    public entry fun initialize_coin<C: key>(
        account: &signer, 
        name: vector<u8>, 
        symbol: vector<u8>,
        ) {
        // Register the coin (Create CoinStore<CoinA> resource)
        coin::register<C>(account);
        
        // Initialize the coin (Create CoinInfo<CoinA> and get 
        // burn, freeze and mint capability's handles to store)
        let (burn_capability, freeze_capability, mint_capability) = 
            coin::initialize<C>(
                account,
                string::utf8(name),
                string::utf8(symbol),
                9,
                false,
            );

        // Mint the coin
        let unique_coin = coin::mint<C>(
            18446744073709551615,
            &mint_capability,
        );

        // Once the coin is minted, deposit it in CoinStore resource
        coin::deposit<C>(signer::address_of(account), unique_coin);

        move_to(
            account, 
            CoinManagement<C> {
                burn_capability,
                freeze_capability,
                mint_capability,
            }
        );
    }

    public entry fun register_unique_coin<C: key>(account: &signer) {
        coin::register<C>(account);
    }

}