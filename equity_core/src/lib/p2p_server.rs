use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use tokio_stream::wrappers::ReceiverStream;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::channel};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message
};
use futures::{SinkExt, StreamExt};
use equity_storage::EquityDatabase;
use equity_types::{Credentials, EquityError, PeerMap, Peer};
use tokio::task::{JoinHandle};
use tracing::info;
use serde::{Deserialize, Serialize};

use ed25519_consensus::{Signature, VerificationKey};



use crate::Error;

pub async fn start_p2p_server(
    p2p_listener: SocketAddr,
    seed_address: SocketAddr,
    db: EquityDatabase,
    peers: PeerMap,
    credentials: Arc<Credentials>
) -> Result<(SocketAddr, JoinHandle<Result<(), EquityError>>), Error> {


    // IF seed address is not given then server will not connect to other servers
    println!("{}", seed_address.to_string());
    if seed_address.to_string() != "0.0.0.0:0".to_string() {
        initialize_network(&seed_address, peers.clone(), &credentials).await;
    }
    
    let try_socket = TcpListener::bind(&p2p_listener).await;
    let listener = try_socket.expect("Failed to bind");
    let bound_addr = listener.local_addr().unwrap();

    info!(target: "equity-core", "Starting P2P Server");
    
    let handle = tokio::spawn(async move {
        // Let's spawn the handling of each connection in a separate task.
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_connection(stream, addr, peers.clone()));
        }
        Ok(())
    });

    info!(target: "equity-core", "P2P Server started at: {}", bound_addr);

    Ok((bound_addr, handle))
}

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr, peers: PeerMap) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (write, mut read) = ws_stream.split();

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = channel(1000);
    let rx = ReceiverStream::new(rx);

    
    if let Some(initial_msg) = read.next().await {
        let initial_msg = initial_msg.unwrap();
        
        let init_message: InitMessage = serde_json::from_str(&initial_msg.into_text().unwrap()).unwrap();

        let mut peer_list: HashMap<SocketAddr, VerificationKey> = HashMap::new();
        
        let mut peers = peers.lock().unwrap();

        let peers_iter = peers.iter();

        // Iterate over everything.
        for (adr, peer) in peers_iter {
            peer_list.insert(adr.clone(), peer.public_key);
        }

        let peer_struct = Peer {
            send: tx,
            public_key: init_message.initiate.public_key
        };

        peers.insert(addr, peer_struct);
    }

    tokio::spawn(
        rx.map(Ok).forward(write)
    );

    while let Some(msg) = read.next().await {
        println!("Received msg");
    }

}

async fn initialize_network(seed_address: &SocketAddr, peers: PeerMap, credentials: &Credentials) {
    let (mut ws_stream, _) = connect_async(seed_address.to_string()).await.expect("Failed to connect");
    
    println!("WebSocket handshake has been successfully completed");

    ws_stream.send(initial_message(credentials)).await.unwrap();

    let msg = ws_stream.next().await;

    let (write, read) = ws_stream.split();

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = channel(1000);
    let rx = ReceiverStream::new(rx);

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

pub struct InitResponse {
    pub peers: HashMap<SocketAddr, VerificationKey>,
    pub hash: String,
    pub signature: Signature
}

pub fn initial_message(credentials: &Credentials) -> Message {

    let initiate: Initiate = Initiate { public_key: credentials.public_key, nonce: credentials.nonce };

    let message_string = serde_json::to_string(&initiate).unwrap();

    let (hash, signature) = credentials.hash_sign(&message_string);
    
    Message::binary(serde_json::to_vec(&InitMessage { initiate, hash, signature }).unwrap())
}


