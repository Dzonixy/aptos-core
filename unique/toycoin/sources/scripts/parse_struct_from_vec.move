script {
    use toycoin::unique::{UniqueResource, get_resources};
    use std::signer;
    use std::debug;

    fun parse_struct_from_vec(account: &signer) {
        get_resources(account);

        // let ps = unique::parse_vector_to_struct(some_vec);
        // unique::parse_vector_to_struct(some_vec);
        
        // debug::print(&some_vec);

        // debug::print(&unique::get_number_u64(&ps));
        // debug::print(&unique::get_number_u8(&ps));
    }
}
