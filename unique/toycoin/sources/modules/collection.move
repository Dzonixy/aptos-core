module toycoin::collection {
    use std::vector;
    use std::signer;

    struct Item has store, drop {
        // we'll think of the properties later
    }

    struct Collection has key, store {
        items: vector<Item>,
    }

    public fun size(account: &signer): u64 acquires Collection {
        let owner = signer::address_of(account);
        let collection = borrow_global<Collection>(owner);

        vector::length(&collection.items)
    }

    public fun add_item(account: &signer) acquires Collection {
        let collection = borrow_global_mut<Collection>(signer::address_of(account));

        vector::push_back(&mut collection.items, Item {});
    }

    public fun exists_at(at: address): bool {
        exists<Collection>(at)
    }

    public fun destroy(account: &signer) acquires Collection {
        let collection = move_from<Collection>(signer::address_of(account));

        let Collection { items: _ } = collection;
    }
}