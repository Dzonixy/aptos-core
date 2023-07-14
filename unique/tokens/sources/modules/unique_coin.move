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

module tokens::escrow {
    use aptos_framework::coin::{Coin, withdraw, deposit, transfer};
    use aptos_framework::type_info;
    use std::signer;

    struct Escrow<phantom T: key> has key {
        offered_coin: Coin<T>,
        maker: address,
        wanted_coin_account_address: address,
        wanted_coin_module_name: vector<u8>,
        wanted_coin_struct_name: vector<u8>,
        wanted_amount: u64,
    }

    public entry fun initialize<C: key>(
        account: &signer, 
        offered_amount: u64,
        wanted_coin_account_address: address,
        wanted_coin_module_name: vector<u8>,
        wanted_coin_struct_name: vector<u8>,
        wanted_amount: u64,
        ) {
        let offered_coin = withdraw<C>(account, offered_amount);

        move_to(
            account, 
            Escrow<C> {
                offered_coin,
                maker: signer::address_of(account),
                wanted_coin_account_address,
                wanted_coin_module_name,
                wanted_coin_struct_name,
                wanted_amount,
            }
        );
    }  

    public entry fun accept<O: key, W: key>(account: &signer, maker: address) acquires Escrow {
        let wanted_coin_info = type_info::type_of<W>();
        let Escrow {
            offered_coin,
            maker,
            wanted_coin_account_address,
            wanted_coin_module_name,
            wanted_coin_struct_name,
            wanted_amount,
        } = move_from<Escrow<O>>(maker);
        assert!(type_info::account_address(&wanted_coin_info) == wanted_coin_account_address, 0);
        assert!(type_info::module_name(&wanted_coin_info) == wanted_coin_module_name, 0);
        assert!(type_info::struct_name(&wanted_coin_info) == wanted_coin_struct_name, 0);

        deposit(signer::address_of(account), offered_coin);
        transfer<W>(account, maker, wanted_amount);
    }
    
 }