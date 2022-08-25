use std::{net::SocketAddr, str::FromStr};

use clap::Parser;
use ed25519_consensus::VerificationKey;
use equity_core::{EquityService, Error};
use equity_storage::EquityDatabase;
use equity_types::Value;
use tracing::info;

use serde_plain;

#[derive(Parser)]
#[clap(name = "equity_core", about = "Equity", version)]
struct CliArgs {
    #[clap(name = "api_listener", default_value = "127.0.0.1:4040")]
    api_listener: String,
    #[clap(name = "p2p_listener", default_value = "127.0.0.1:5050")]
    p2p_listener: String,
    #[clap(name = "seed_address", default_value = "0.0.0.0:0000")]
    seed_address: String,
    #[clap(name = "seed_public_key")]
    seed_public_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = CliArgs::parse();
    let api_listener = SocketAddr::from_str(&args.api_listener)?;
    let p2p_listener = SocketAddr::from_str(&args.p2p_listener)?;
    let seed_address = SocketAddr::from_str(&args.seed_address)?;
    
    let seed_public_key;

    if &args.seed_address != "0.0.0.0:0000" {
        let public_key_result = serde_plain::from_str::<VerificationKey>(&args.seed_public_key);
        if let Ok(key) = public_key_result {
            seed_public_key = key;
        };
    }

    initialize_logger();
    info!(target: "equity-core", "Initializing equity-core");

    // todo: read from config and initialize correct db
    let db = EquityDatabase::in_memory();
    genesis_data(&db);

    let service = EquityService::new(api_listener, p2p_listener, seed_address, seed_public_key, db).await?;

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
