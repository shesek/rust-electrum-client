extern crate electrum_client;

use electrum_client::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::new("kirsche.emzy.de:50001").await.unwrap();
    let res = client.server_features().await;
    println!("{:#?}", res);
}
