use clap::Parser;
use equity_client::EquityClient;
use credentials::Keys;
use tracing::info;
use std::{thread, time};

#[derive(Parser)]
#[clap(name = "equity_cli", about = "Equity", version)]
struct CliArgs {
    #[clap(
        name = "endpoint",
        default_value = "ws://localhost:4040",
        long = "endpoint"
    )]
    endpoint: String,

    #[clap(subcommand)]
    command: CliCommand,
}

#[derive(Parser)]
enum CliCommand {
    Health,
    Transaction {
        key_domain: u64,
        value_range: u64,
        iterations: u8,
    },
}

#[tokio::main]
pub async fn main() {
    let args = CliArgs::parse();
    initialize_logger();

    let mut client = EquityClient::new(&args.endpoint, Keys::Empty).await.unwrap();
    
    match &args.command {
        CliCommand::Health => {
            let response = client.health().await;
            info!("Health Response is: {:?}", response);
        }
        CliCommand::Transaction {
            key_domain,
            value_range,
            iterations,
        } => {
            println!("DB Key Domain: {:?}", key_domain);
            println!("DB Key Range: {:?}", value_range);
            println!("Iterations: {:?}", iterations);
            let tester = client.test_transaction(key_domain, value_range, iterations);
            let transaction = client.credentials.transaction(tester).await.unwrap();
            client.send_transaction(transaction).await;
            thread::sleep(time::Duration::from_secs(5));
            info!("Transaction submitted");
        }
    }
}

fn initialize_logger() {
    let sub = tracing_subscriber::fmt::Subscriber::builder().with_writer(std::io::stderr);
    sub.with_ansi(true)
        .with_level(true)
        .with_line_number(true)
        .with_file(true)
        .init();
}
