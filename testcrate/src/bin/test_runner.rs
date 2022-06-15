use std::time::Duration;

use clap::Parser;
use common::test_mode::TestMode;
use equity_client::EquityClient;

const TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Parser)]
#[clap(version)]
pub struct CliArgs {
    #[clap(arg_enum)]
    pub test_mode: TestMode,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    match args.test_mode {
        TestMode::Health => {
            let client = EquityClient::new("http://equity_core:4040").unwrap();
            client.wait_for_healthy(TIMEOUT).await.unwrap();
            dbg!(client.health().await.unwrap());
            assert!(client.health().await.unwrap().up);
        }
    }
}
