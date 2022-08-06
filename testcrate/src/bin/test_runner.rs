use std::time::Duration;

use clap::Parser;
use common::test_mode::TestMode;
use equity_client::EquityClient;
use equity_types::{EquityAddressResponse, Value};

const TIMEOUT: Duration = Duration::from_secs(15);

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
        mode @ (TestMode::Health | TestMode::GetResponse) => {
            let client = EquityClient::new("http://equity_core:4040");
            client.wait_for_healthy(TIMEOUT).await.unwrap();
            match mode {
                TestMode::Health => {
                    dbg!(client.health().await.unwrap());
                    assert!(client.health().await.unwrap().up);
                }
                TestMode::GetResponse => {
                    dbg!(client.get_account_details("testkey").await.unwrap());
                    assert_eq!(
                        client.get_account_details("testkey").await.unwrap(),
                        EquityAddressResponse {
                            owner: "testkey".to_owned(),
                            value: Value(1337)
                        }
                    );
                }
            }
        }
    }
}
