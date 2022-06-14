use std::time::Duration;

use equity_client::EquityClient;

const TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() {
    let client = EquityClient::new("http://equity_core:4040").unwrap();
    client.wait_for_healthy(TIMEOUT).await.unwrap();
    dbg!(client.health().await.unwrap());
    assert!(client.health().await.unwrap().up);
}
