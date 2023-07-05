script {
    use toycoin::unique::add_one_number;

    fun add_one_number(account: &signer) {
        add_one_number(account);
    }
}