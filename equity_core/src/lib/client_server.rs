use std::{
    net::SocketAddr,
    sync::Arc,
};


use ed25519_consensus::{Signature, VerificationKey};
use equity_storage::EquityDatabase;
use equity_types::{
    Credentials, EquityAddressResponse, EquityError, FullMessage, HealthResponse,
    PostTransactionResponse, ClientCommand, TransactionBody
};
use equity_p2p::PeerMap;
use futures::{SinkExt, StreamExt};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Sender},
    task::{spawn_blocking, JoinHandle}
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Peer {
    address: SocketAddr,
    public_key: VerificationKey,
}

pub async fn start_client_server(
    api_listener: SocketAddr,
    db: EquityDatabase,
    peers: PeerMap,
    credentials: Arc<Credentials>,
) -> Result<(SocketAddr, JoinHandle<Result<(), EquityError>>), Error> {
    
    let try_socket = TcpListener::bind(&api_listener).await;
    let listener = try_socket.expect("Failed to bind");
    let bound_addr = listener.local_addr().unwrap();

    let (tx, rx) = tokio::sync::oneshot::channel();
    
    info!(target: "equity-core", "Starting WS Client Server");
    let handle = tokio::spawn(async move {
        
        let _ = tx.send(());

        // Let's spawn the handling of each connection in a separate task.
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_connection(
                stream,
                addr,
                peers.clone(),
                credentials.clone(),
                db.clone()
            ));
        }
        Ok(())
    });

    let _ = rx.await;

    info!(target: "equity-core", "WS Client Server started at: {}", bound_addr);

    Ok((bound_addr, handle))
}

async fn handle_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
    peers: PeerMap,
    credentials: Arc<Credentials>,
    db: EquityDatabase
) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (write, mut read) = ws_stream.split();

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = channel(1000);
    
    let rx = ReceiverStream::new(rx);

    tokio::spawn(rx.map(Ok).forward(write));

    while let Some(Ok(msg)) = read.next().await {
        let tx_clone = tx.clone();
        let command: ClientCommand = serde_json::from_slice(&msg.into_data()).unwrap();
        tokio::spawn(
            client_switch(
                command, 
                tx_clone, 
                peers.clone(), 
                credentials.clone(), 
                db.clone()
        ));
    }
}

async fn client_switch(
    client_command: ClientCommand, 
    sender: Sender<Message>,
    peers: PeerMap,
    credentials: Arc<Credentials>,
    db: EquityDatabase
) {
    match client_command {
        ClientCommand::Health { } => {
            let response = health();
            sender.send(Message::binary(serde_json::to_vec(&response).expect("msg does not have serde serialize trait")));
        },
        ClientCommand::Transaction{ body, hash, signature } => {
            let response = transaction(db, body, hash, signature);
            sender.send(Message::binary(serde_json::to_vec(&response).expect("msg does not have serde serialize trait")));
        }
    }
}

fn health() -> HealthResponse {
    info!(target = "equity-core", "Health API");
    HealthResponse { up: true }
}

async fn transaction(
    db: EquityDatabase,
    body: TransactionBody,
    hash: String,
    signature: Signature
) -> PostTransactionResponse {
    info!(target = "equity-core", "Transaction API");

    // Check database if Mapping [hash -> tx_record] exists
    // If value exists revert transaction

    if let Ok(Some(_value)) = db.get(hash.as_bytes()) {
        return PostTransactionResponse {
            success: false,
            msg: "Revert: TX already exists".to_string(),
        }
    };

    // Verify signature
    // If signature is not verified then revert transaction

    if let Ok(Err(e)) = spawn_blocking(move || verify_body(&body, &hash, &signature)).await {
        return Ok(Json(PostTransactionResponse {
            success: false,
            msg: e.to_string(),
        }))
    }

    // Post transaction record to db
    let payload_entry = payload.clone();
    // let payload_hash = payload.hash;

    if let Ok(None) = state.set(&payload.hash, payload_entry) {
        return Ok(Json(PostTransactionResponse {
            success: true,
            msg: "Transaction entry recorded to db".to_string(),
        }))
    };

    Ok(Json(PostTransactionResponse {
        success: false,
        msg: "Transaction not recorded to db".to_string(),
    }))
}

fn verify_body(body: &TransactionBody, hash: &String, signature: &Signature) -> Result<(), ed25519_consensus::Error> {
    let mut digest: Sha512 = Sha512::new();

    digest.update(serde_json::to_string(&payload.body).unwrap());

    let digest_string: String = format!("{:X}", digest.clone().finalize());

    payload
        .body
        .public_key
        .verify(&payload.signature, digest_string.as_bytes())
}




