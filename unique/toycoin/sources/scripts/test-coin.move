script {
    use std::vector;
    use 0x2::Coin;

    fun main() {
        let coin = Coin::mint(100);
        let v = &mut vector::empty<u64>();
        vector::push_back(v, 10);

        Coin::burn(coin);
    }
}