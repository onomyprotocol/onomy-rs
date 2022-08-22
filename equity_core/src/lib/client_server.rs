use std::net::SocketAddr;

use ed25519_consensus::VerificationKey;

use equity_types::{

    BroadcastMsg, EquityError, HealthResponse,
    PostTransactionResponse, ClientMsg, SignInput, Transaction, TransactionCommand
};

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Sender},
    task::JoinHandle
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
        let command: ClientMsg = serde_json::from_slice(&msg.into_data()).unwrap();
        tokio::spawn(
            client_switch(
                command, 
                tx_clone, 
                context.clone()
        ));
    }
}

async fn client_switch(
    client_command: ClientMsg, 
    sender: Sender<Message>,
    context: Context
) {
    // let client_command_clone = client_command.clone();
    match &client_command {
        ClientMsg::Health { } => {
            let response = health();
            sender
                .send(Message::binary(serde_json::to_vec(&response)
                .expect("msg does not have serde serialize trait"))).await.unwrap();
        },
        ClientMsg::Transaction(transaction) => {
            if let Error = verify_signature(&transaction) {
                return
            }
            match &transaction.command {
                TransactionCommand::SetValues { keys_values } => {
                    let response = set_values(
                        &context,
                        &transaction
                    ).await;
                    sender
                        .send(Message::binary(serde_json::to_vec(&response)
                        .expect("msg does not have serde serialize trait"))).await.unwrap();
                },
                TransactionCommand::SetValidator { ws } => {
                     
                    // 1) Connect
                    let connection = peer_connection(ws, &context).await;
                    
                    // 2) Once Connected - Initiate BRB
                    // At end of BRB, then peer connection is added
                    // to the validator list.  So, BRB needs to store this connection
                    // The task will need to hold the Command and anything else related
                    match connection {
                        Ok(()) => {
                            context.brb.initiate(hash, body.public_key,  BroadcastMsg::Transaction(Transaction {
                                body: body.clone(),
                                hash: hash.clone(),
                                signature: *signature
                            }));
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

async fn set_values(
    context: &Context,
    transaction: &Transaction
) -> PostTransactionResponse {
    info!(target = "equity-core", "Transaction API");

    // Check database if Mapping [hash -> tx_record] exists
    // If value exists revert transaction. There are no duplicates allowed
    if let Ok(Some(_value)) = context.db.get::<Transaction>(&transaction.hash.as_bytes()) {
        return PostTransactionResponse {
            success: false,
            msg: "Revert: TX already exists".to_string(),
        }
    };

    // Post transaction record to db
    if let Ok(None) = context.db.set(&transaction.hash, transaction.clone()) {
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


fn verify_signature(transaction: &Transaction) -> Result<(), Error> {
    let mut digest: Sha512 = Sha512::new();

    digest.update(serde_json::to_string(&SignInput{
        input: serde_json::to_string(&transaction.command).unwrap(),
        salt: transaction.salt
    }).unwrap());

    let hash: String = format!("{:X}", digest.finalize());

    transaction.public_key.verify(&transaction.signature, &hash.as_bytes());
    
    Ok(())
}




