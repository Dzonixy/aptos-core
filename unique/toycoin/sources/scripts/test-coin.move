script {
    use std::vector;
    use 0x2::UniqueToken;

    fun main() {
        let unique_token = UniqueToken::mint(100);
        let v = &mut vector::empty<u64>();
        vector::push_back(v, 10);

        UniqueToken::burn(unique_token);
    }
}