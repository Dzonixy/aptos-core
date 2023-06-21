script {
    use std::vector;
    use 0x2::UniqueToken;

    fun main(a: u64) {
        let unique_token = UniqueToken::mint(100);
        let v = &mut vector::empty<u64>();
        vector::push_back(v, 10);
        a;
        UniqueToken::burn(unique_token);
    }
}