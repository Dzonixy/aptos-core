script {
    use std::vector;
    use toycoin::unique::gimme_five;
    use toycoin::unique;
    use std::debug;
    use std::event;
    use toycoin::unique::new_unique;

    fun main(account: &signer, a: u64, b: u64) {
        let v = &mut vector::empty<u64>();
        vector::push_back(v, 10);

        unique::sum(3, 2);
        let five = gimme_five();

        new_unique(account);

        debug::print(&b"hello there");
        assert!(a == b, 100);
        assert!(gimme_five() == 5, 101);
    }
}