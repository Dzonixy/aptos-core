script {
    use toycoin::unique;
    use std::debug;

    fun parse_struct_from_vec(some_vec: vector<u8>) {
        let ps = unique::parse_vector_to_struct(some_vec);

        // debug::print(&unique::get_number_u64(&ps));
        // debug::print(&unique::get_number_u8(&ps));
    }
}

    




