use std::net::SocketAddr;

use ed25519_consensus::{Signature, VerificationKey};

use equity_types::{
    EquityError, HealthResponse,
    PostTransactionResponse, ClientCommand, TransactionBody
};

use equity_types::TransactionBody::{ SetValues, SetValidator };

use equity_types::MsgType;

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Sender},
    task::{spawn_blocking, JoinHandle}
};

use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::tungstenite::Message;
use tracing::info;

use crate::error::Error;

use crate::service::Context;
use crate::p2p_server::peer_connection;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Peer {
    address: SocketAddr,
    public_key: VerificationKey,
}

pub async fn start_client_server(
    api_listener: SocketAddr,
    context: Context
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
                context.clone()
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
    context: Context
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
                context.clone()
        ));
    }
}

async fn client_switch(
    client_command: ClientCommand, 
    sender: Sender<Message>,
    context: Context
) {
    match client_command {
        ClientCommand::Health { } => {
            let response = health();
            sender
                .send(Message::binary(serde_json::to_vec(&response)
                .expect("msg does not have serde serialize trait"))).await.unwrap();
        },
        ClientCommand::Transaction{ body, hash, signature } => {
            match body {
                SetValues { public_key, nonce, keys_values } => {
                    if let Error = verify_signature(&body, public_key, &signature) {
                        return
                    }
                    let response = transaction(context, body, hash, signature).await;
                    sender
                        .send(Message::binary(serde_json::to_vec(&response)
                        .expect("msg does not have serde serialize trait"))).await.unwrap();
                },
                SetValidator { public_key, nonce, ws } => {
                    if let Error = verify_signature(&body, public_key, &signature) {
                        return
                    }
                    // 1) Connect
                    let connection = peer_connection(ws, context).await;
                    
                    // 2) Once Connected - Initiate BRB
                    // At end of BRB, then peer connection is added
                    // to the validator list.  So, BRB needs to store this connection
                    // The task will need to hold the Command and anything else related
                    match connection {
                        Ok(()) => {
                            context.brb.initiate(hash, public_key,  MsgType::Client(client_command));
                        },
                        Error => {
                            return
                        }
                    }
                }
            }
        }
    }
}

fn health() -> HealthResponse {
    info!(target = "equity-core", "Health API");
    HealthResponse { up: true }
}

async fn transaction(
    context: Context,
    body: TransactionBody,
    hash: String,
    signature: Signature
) -> PostTransactionResponse {
    info!(target = "equity-core", "Transaction API");

    // Check database if Mapping [hash -> tx_record] exists
    // If value exists revert transaction. There are no duplicates allowed

    if let Ok(Some(_value)) = context.db.get::<TransactionBody>(&hash.as_bytes()) {
        return PostTransactionResponse {
            success: false,
            msg: "Revert: TX already exists".to_string(),
        }
    };

    // Pre-Verify Transaction
    // 1) Verify Signature
    // 2) Verify Transaction Enabled by State
    // If transaction is not verified then revert transaction
    let body_verify = body.clone();
    let hash_verify = hash.clone();
    let signature_verify = signature.clone();

    if let Ok(Err(e)) = spawn_blocking(move || verify_body(&body_verify, &hash_verify, &signature_verify)).await {
        return PostTransactionResponse {
            success: false,
            msg: e.to_string(),
        }
    }

    // Post transaction record to db
    if let Ok(None) = context.db.set(&hash, body) {
        return PostTransactionResponse {
            success: true,
            msg: "Transaction entry recorded to db".to_string(),
        }
    };

    PostTransactionResponse {
        success: false,
        msg: "Transaction not recorded to db".to_string(),
    }
}


fn verify_signature(body: &TransactionBody, public_key: VerificationKey, signature: &Signature) -> Result<(), Error> {
    let mut digest: Sha512 = Sha512::new();

    digest.update(serde_json::to_string(&body).unwrap());

    let digest_string: String = format!("{:X}", digest.clone().finalize());

    
    public_key.verify(signature, digest_string.as_bytes());
    
    Ok(())
}




