use aptos_framework::BuiltPackage;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join(&format!("../{}/", "toycoin"));
    BuiltPackage::build(path, aptos_framework::BuildOptions::default()).unwrap();
}
