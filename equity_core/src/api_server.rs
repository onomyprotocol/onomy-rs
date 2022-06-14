use std::net::{Ipv4Addr, SocketAddr, TcpListener};

use axum::{extract::Path, routing, Extension, Router};
use equity_storage::EquityDatabase;
use equity_types::{EquityAddressResponse, HealthResponse};
use hyper::StatusCode;
use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::info;

use crate::borsh::Borsh;

pub async fn start_api_server(
    db: EquityDatabase,
) -> Result<(SocketAddr, JoinHandle<Result<(), EquityError>>), std::io::Error> {
    let router = Router::new()
        .route("/health", routing::get(health))
        .route("/address/:key", routing::get(get_address))
        .layer(Extension(db));

    let listener = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 4040);
    let listener = TcpListener::bind(&listener)?;
    let bound_addr = listener.local_addr().unwrap();

    let (tx, rx) = tokio::sync::oneshot::channel();

    info!(target: "equity-core", "Starting API Server");
    let handle = tokio::spawn(async move {
        let server = axum::Server::from_tcp(listener)
            .unwrap()
            .serve(router.into_make_service());

        let _ = tx.send(());

        server.await.map_err(Into::into)
    });

    let _ = rx.await;
    info!(target: "equity-core", "API Server started at: {}", bound_addr);

    Ok((bound_addr, handle))
}

async fn health() -> Borsh<HealthResponse> {
    info!(target = "equity-core", "Health API");
    Borsh(HealthResponse { up: true })
}

async fn get_address(
    Path(key): Path<String>,
    Extension(state): Extension<EquityDatabase>,
) -> Result<Borsh<EquityAddressResponse>, StatusCode> {
    info!(
        target = "equity-core",
        "Get Address API: address is: `{}`", key
    );

    match state.get(&key.bytes().collect::<Vec<_>>()) {
        Ok(value) => {
            let response = Borsh(EquityAddressResponse {
                owner: key,
                value: value.unwrap(),
            });

            Ok(response)
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Error)]
pub enum EquityError {
    #[error("An api server error occurred {0}")]
    ApiServer(#[from] hyper::Error),
}
