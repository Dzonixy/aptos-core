script {
    use std::vector;
    use toycoin::unique::gimme_five;
    use toycoin::unique;
    use std::debug;
    use std::event;
    use toycoin::unique::new_unique;
    use std::signer;
    use std::string;

    fun main(account: &signer, a: u64, b: u64, number: u64, c: vector<u8>) {
        let v = &mut vector::empty<u64>();
        vector::push_back(v, 10);

        unique::sum(3, 2);
        let five = gimme_five();

        new_unique(account, number, c);

        debug::print(&signer::address_of(account));
        assert!(a == b, 100);
        assert!(gimme_five() == 5, 101);
    }
}