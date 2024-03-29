use clap::Parser;
use equity_client::EquityClient;
use tracing::{error, info};

#[derive(Parser)]
#[clap(name = "equity_cli", about = "Equity", version)]
struct CliArgs {
    #[clap(
        name = "endpoint",
        default_value = "http://localhost:4040",
        long = "endpoint"
    )]
    endpoint: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    Account {
        address: String,
    },
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

    let mut client = EquityClient::new(&args.endpoint).unwrap();
    match &args.command {
        Command::Account { address } => match client.get_account_details(address).await {
            Ok(response) => info!("{:?}", response),
            Err(e) => error!("{:?}", e),
        },
        Command::Health => {
            let response = client.health().await.unwrap();
            info!("Health Response is: {:?}", response);
        }
        Command::Transaction {
            key_domain,
            value_range,
            iterations,
        } => {
            println!("DB Key Domain: {:?}", key_domain);
            println!("DB Key Range: {:?}", value_range);
            println!("Iterations: {:?}", iterations);
            client.noncer();
            let tester = client.test_transaction(key_domain, value_range, iterations);
            let transaction = client.create_transaction(&tester);
            let response = client.post_transaction(transaction).await.unwrap();
            info!("Transaction Response is: {:?}", response);
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
