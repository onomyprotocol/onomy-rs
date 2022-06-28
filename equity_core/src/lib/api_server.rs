use std::net::{SocketAddr, TcpListener};

use axum::{extract::Path, routing, Extension, Router};
use equity_storage::EquityDatabase;
use equity_types::{EquityAddressResponse, HealthResponse, PostTransactionResponse};
use hyper::StatusCode;
use tokio::task::JoinHandle;
use tracing::info;

use crate::{borsh::Borsh, Error};

pub async fn start_api_server(
    listener: SocketAddr,
    db: EquityDatabase,
) -> Result<(SocketAddr, JoinHandle<Result<(), EquityError>>), Error> {
    let router = Router::new()
        .route("/health", routing::get(health))
        .route("/address/:key", routing::get(get_address))
        .route("/transaction", routing::get(transaction))
        .layer(Extension(db));

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

async fn transaction() -> Borsh<PostTransactionResponse> {
    info!(target = "equity-core", "Transaction API");
    Borsh(PostTransactionResponse { success: true })
}

// TODO should we use some binary instead of a path?

async fn get_address(
    Path(key): Path<String>,
    Extension(state): Extension<EquityDatabase>,
) -> Result<Borsh<EquityAddressResponse>, StatusCode> {
    info!(
        target = "equity-core",
        "Get Address API: address is: `{}`", key
    );

    match state.get(key.as_bytes()) {
        Ok(Some(value)) => {
            let response = Borsh(EquityAddressResponse { owner: key, value });
            Ok(response)
        }
        Ok(None) => {
            info!("not found");
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            info!("error: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EquityError {
    #[error("An api server error occurred {0}")]
    ApiServer(#[from] hyper::Error),
}
