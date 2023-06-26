script {
    use std::vector;

    fun main(a: u64, b: u64) {
        let v = &mut vector::empty<u64>();
        vector::push_back(v, 10);

        assert!(a == b, 100);
    }
}