use std::{net::SocketAddr, str::FromStr};

use clap::Parser;
use equity_core::{EquityService, Error};
use equity_storage::EquityDatabase;
use equity_types::Value;
use tracing::info;

#[derive(Parser)]
#[clap(name = "equity_core", about = "Equity", version)]
struct CliArgs {
    #[clap(name = "api_listener", default_value = "127.0.0.1:4040")]
    api_listener: String,
    #[clap(name = "p2p_listener", default_value = "127.0.0.1:5050")]
    p2p_listener: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = CliArgs::parse();
    let api_listener = SocketAddr::from_str(&args.api_listener)?;
    let p2p_listener = SocketAddr::from_str(&args.p2p_listener)?;
    initialize_logger();
    info!(target: "equity-core", "Initializing equity-core");

    // todo: read from config and initialize correct db
    let db = EquityDatabase::in_memory();
    genesis_data(&db);

    let service = EquityService::new(api_listener, p2p_listener, db).await?;

    service.run().await;

    Ok(())
}

fn initialize_logger() {
    let sub = tracing_subscriber::fmt::Subscriber::builder().with_writer(std::io::stderr);

    sub.with_ansi(false)
        .with_level(true)
        .with_line_number(true)
        .with_file(true)
        .init();
}

fn genesis_data(db: &EquityDatabase) {
    let _ = db.set("testkey", Value(1337));
}
