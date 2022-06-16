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
    Account { address: String },
    Health,
}

#[tokio::main]
pub async fn main() {
    let args = CliArgs::parse();
    initialize_logger();

    let client = EquityClient::new(&args.endpoint).unwrap();
    match &args.command {
        Command::Account { address } => match client.get_account_details(address).await {
            Ok(response) => info!("{:?}", response),
            Err(e) => error!("{:?}", e),
        },
        Command::Health => {
            let response = client.health().await.unwrap();
            info!("Health Response is: {:?}", response);
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
