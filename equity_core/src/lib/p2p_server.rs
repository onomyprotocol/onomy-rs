use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

use equity_storage::EquityDatabase;
use equity_types::{EquityError};
use tokio::task::{JoinHandle};
use tracing::info;
use serde::{Deserialize, Serialize};


use ed25519_consensus::{VerificationKey};
use crate::{Error};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Peer {
    address: SocketAddr,
    public_key: VerificationKey
}

pub async fn start_p2p_server(
    p2p_listener: SocketAddr,
    db: EquityDatabase,
) -> Result<(SocketAddr, JoinHandle<Result<(), EquityError>>), Error> {
    
    let try_socket = TcpListener::bind(&p2p_listener).await;
    let listener = try_socket.expect("Failed to bind");
    let bound_addr = listener.local_addr().unwrap();

    info!(target: "equity-core", "Starting P2P Server");
    
    let handle = tokio::spawn(async move {
        // Let's spawn the handling of each connection in a separate task.
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_connection(stream, addr));
        }
        Ok(())
    });

    info!(target: "equity-core", "P2P Server started at: {}", bound_addr);

    Ok((bound_addr, handle))
}

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);
}

