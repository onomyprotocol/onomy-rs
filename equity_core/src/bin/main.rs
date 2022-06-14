use std::{net::SocketAddr, str::FromStr};

use clap::Parser;
use equity_core::{EquityService, Error};
use equity_storage::EquityDatabase;
use tracing::info;

#[derive(Parser)]
#[clap(name = "equity_core", about = "Equity", version)]
struct CliArgs {
    #[clap(name = "listener", default_value = "0.0.0.0:4040", long = "listener")]
    listener: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = CliArgs::parse();
    let listener = SocketAddr::from_str(&args.listener)?;
    initialize_logger();
    info!(target: "equity-core", "Initializing equity-core");

    // todo: read from config and initialize correct db
    let db = EquityDatabase::in_memory();
    genesis_data(&db);

    let service = EquityService::new(listener, db).await?;

    service.run().await;

    Ok(())
}

fn initialize_logger() {
    let sub = tracing_subscriber::fmt::Subscriber::builder().with_writer(std::io::stderr);

    sub.with_ansi(false)
        .with_level(true)
        .with_line_number(true)
        .init();
}

fn genesis_data(db: &EquityDatabase) {
    let _ = db.insert("elvis", 100_000_u64);
    let _ = db.insert("charles", 100_000);
    let _ = db.insert("isaac", 100_000);
}
