use std::time::Duration;

use equity_client::EquityClient;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let client = EquityClient::new("http://equity_core").unwrap();
    dbg!(client.health().await.unwrap());
    sleep(Duration::from_secs(50)).await;
}
