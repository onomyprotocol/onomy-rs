use clap::Parser;
use client::EquityClient;
use tracing::{error, info};
mod client;

#[derive(Parser)]
#[clap(name = "equity-cli", about = "Equity", version)]
struct CliArgs {
    #[clap(name = "endpoint", default_value = "127.0.0.1:4040", long = "endpoint")]
    endpoint: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    Account { address: String },
    Health,
}

impl CliArgs {
    async fn exec(&self) {
        let client = EquityClient::new("http://".to_owned() + &self.endpoint).unwrap();

        match &self.command {
            Command::Account { address } => {
                match client.get_account_details(address.to_owned()).await {
                    Ok(response) => info!("{:?}", response),
                    Err(e) => error!("{:?}", e),
                }
            }
            Command::Health => {
                let response = client.health().await.unwrap();
                info!("Health Response is: {:?}", response);
            }
        }
    }
}

fn main() {
    initialize_logger();
    futures::executor::block_on(CliArgs::parse().exec());
}

fn initialize_logger() {
    let sub = tracing_subscriber::fmt::Subscriber::builder().with_writer(std::io::stderr);

    sub.with_ansi(true)
        .with_level(true)
        .with_line_number(true)
        .init();
}
