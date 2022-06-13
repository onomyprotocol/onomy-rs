use equity_client::EquityClient;

#[tokio::main]
async fn main() {
    let client = EquityClient::new("http://host_equity").unwrap();
    dbg!(client.health().await.unwrap());
}
