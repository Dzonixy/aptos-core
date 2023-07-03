script {
    use std::vector;
    use toycoin::unique::gimme_five;
    use toycoin::unique;

    fun main(a: u64, b: u64) {
        let v = &mut vector::empty<u64>();
        vector::push_back(v, 10);

        unique::sum(3, 2);
        gimme_five();

        assert!(a == b, 100);
        assert!(gimme_five() == 5, 101);
    }
}