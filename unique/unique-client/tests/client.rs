use aptos_sdk::rest_client::Client;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn client_build() {
    let _client = Client::new(url::Url::parse("http://0.0.0.0:8080").unwrap());
}
