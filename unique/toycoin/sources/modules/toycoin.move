module toycoin::unique {
    use std::signer;
    use std::copyable_any;
    use std::debug;

    struct ParsedStruct has drop, store, copy {
        number_u64: u64,
        number_u8: u8,
    }
    
    struct UniqueResource has key, store {
        number: u64,
        msg: vector<u8>,
        unique_data: copyable_any::Any,
    }

    struct Unique has drop, copy {
        value: u64,
    }

    public entry fun new_unique(account: &signer, number: u64, msg: vector<u8>) {
        let data = ParsedStruct {
            number_u64: 1024,
            number_u8: 8,
        };

        move_to<UniqueResource>(account, 
        UniqueResource {
            number,
            msg,
            unique_data: copyable_any::pack<ParsedStruct>(data),
        });
    }

    public fun get_resources(account: &signer) acquires UniqueResource {
        let unique_resource = borrow_global<UniqueResource>(signer::address_of(account));
        let ps = copyable_any::unpack<ParsedStruct>(unique_resource.unique_data);
        debug::print(unique_resource);
        debug::print(&ps);
    }

    public fun add_one_number(account: &signer) acquires UniqueResource {
        let unique_recource = borrow_global_mut<UniqueResource>(signer::address_of(account));
        unique_recource.number = unique_recource.number + 1;
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

    public fun parse_vector_to_struct(vec: vector<u8>) {
    // public fun parse_vector_to_struct(vec: vector<u8>): ParsedStruct {
        from_bytes<ParsedStruct>(vec);
        // from_bytes<ParsedStruct>(vec)
    }

    public fun get_number_u64(ps: &ParsedStruct): u64 {
       *&ps.number_u64
    }

    public fun get_number_u8(ps: &ParsedStruct): u8 {
        *&ps.number_u8
    }

    public native fun from_bytes<T>(bytes: vector<u8>): T;
}
