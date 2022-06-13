mod api_server;
mod borsh;
mod service;

use equity_storage::EquityDatabase;
use service::EquityService;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    initialize_logger();
    info!(target: "equity-core", "Initializing equity-core");

    // todo: read from config and initialize correct db
    let db = EquityDatabase::in_memory();
    genesis_data(&db);

    let service = EquityService::new(db).await?;

    service.run().await;

    Ok(())
}

fn initialize_logger() {
    let sub = tracing_subscriber::fmt::Subscriber::builder().with_writer(std::io::stderr);

    sub.with_ansi(true)
        .with_level(true)
        .with_line_number(true)
        .init();
}

fn genesis_data(db: &EquityDatabase) {
    let _ = db.insert("elvis", 100_000_u64);
    let _ = db.insert("charles", 100_000);
    let _ = db.insert("isaac", 100_000);
}
