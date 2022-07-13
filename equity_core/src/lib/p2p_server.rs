use std::net::SocketAddr;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::unbounded_channel};
use tokio_tungstenite::{connect_async};
use futures::StreamExt;
use equity_storage::EquityDatabase;
use equity_types::{Credentials, EquityError, PeerMap, Peer};
use tokio::task::{JoinHandle};
use tracing::info;
use serde::{Deserialize, Serialize};

use ed25519_consensus::{Signature, VerificationKey};


pub use tokio_tungstenite::tungstenite::protocol::Message;

use crate::{Error};

pub async fn start_p2p_server(
    p2p_listener: SocketAddr,
    seed_address: SocketAddr,
    db: EquityDatabase,
    peers: PeerMap,
    credentials: Credentials
) -> Result<(SocketAddr, JoinHandle<Result<(), EquityError>>), Error> {


    // If seed address given then network is already initialize
    // IF seed address is not given then server will not connect to other servers
    if seed_address.to_string() != "0.0.0.0".to_string() {
        initialize_network(&seed_address, peers.clone());
    }
    
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

async fn initialize_network(seed_address: &SocketAddr, peers: PeerMap) {
    let (mut ws_stream, _) = connect_async(seed_address.to_string()).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    // ws_stream.send();

    let msg = ws_stream.next().await;

    let (write, read) = ws_stream.split();

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);

    tokio::spawn(
        rx.map(Ok).forward(write)
    );
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Initiate {
    pub public_key: VerificationKey,
    pub nonce: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct InitMessage {
    pub initiate: Initiate,
    pub hash: String,
    pub signature: Signature,
}

pub fn initial_message(credentials: Credentials, message: &Initiate) -> InitMessage {

    let message_string = serde_json::to_string(message).unwrap();

    let (digest_string, signature) = credentials.hash_sign(&message_string);
    

    InitMessage {
        initiate: message.clone(),
        hash: digest_string,
        signature,
    }
}


