use std::{ net::SocketAddr, collections::HashMap };

use ed25519_consensus::VerificationKey;
use equity_types::{ EquityError, PeerMsg, TransactionCommand, Broadcast::{ Init, Echo, Ready }, socket_to_ws, SignedMsg };
use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Sender},
    task::JoinHandle,
};

use serde_json::Value;

use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;
use crate::service::Context;
use crate::error::Error;

pub async fn start_p2p_server(
    p2p_listener: SocketAddr,
    seed_address: SocketAddr,
    context: Context
) -> Result<(SocketAddr, JoinHandle<Result<(), EquityError>>), Error> {
    
    let try_socket = TcpListener::bind(&p2p_listener).await;
    let listener = try_socket.expect("Failed to bind");
    let bound_addr = listener.local_addr().unwrap();

    info!(target: "equity-core", "Starting P2P Server");

    let context_handle_connection = context.clone();

    let handle = tokio::spawn(async move {
        // Let's spawn the handling of each connection in a separate task.
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_connection(
                stream,
                addr,
                context_handle_connection.clone()
            ));
        }
        Ok(())
    });

    info!(target: "equity-core", "P2P Server started at: {}", bound_addr);

    // Send init validator TX to Seed Peer.
    if seed_address.to_string() != *"0.0.0.0:0" {
        context.client.send_transaction(
            context.clone().client.sign_transaction(
                &TransactionCommand::SetValidator { ws: p2p_listener }
            ).await
        );
    }

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

    if let Some(initial_msg) = read.next().await {
        let initial_msg = initial_msg.unwrap();

        if let PeerMsg::PeerInit { peer_list } =
            serde_json::from_str(&initial_msg.into_text().unwrap()).unwrap() {

            }

        // Add Peer to list
    }

    tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            let tx_clone = tx.clone();
            let context = context.clone();
            // Need validation
            let command: PeerMsg = serde_json::from_slice(&msg.into_data()).unwrap();
            tokio::spawn(async move {
                p2p_switch(
                    command, 
                    tx_clone, 
                    context
                )
            });
        }
    });
    

    // let mut peers = context.peers.lock().unwrap();

    // peers.remove(&listener);
}

fn key_to_string(key: &VerificationKey) -> Result<String, serde_json::Error> {
    let result = serde_json::from_slice::<Value>(&key.to_bytes());
    match result {
        Ok(val) => Ok(val.to_string()),
        Err(e) => Err(e)
    }
}

pub async fn peer_connection(peer_address: &SocketAddr, peer_public_key: &VerificationKey, context: &Context) -> Result<(), Error> {
    let (mut ws_stream, _) = connect_async(socket_to_ws(peer_address))
        .await
        .expect("Failed to connect");

    println!("WebSocket handshake has been successfully completed");
    
    let peer_list = 
    context.peers
        .lock()
        .expect("Lock poisoned")
        .keys()
        .map(|key| key.clone())
        .collect::<Vec<VerificationKey>>();


    // Send ClientMsg
    ws_stream
        .send(
            Message::binary(
                serde_json::to_vec(&sign_msg(
                &PeerMsg::PeerInit { peer_list }
            ).await).unwrap()
        )).await;

    let (write, mut read) = ws_stream.split();

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = channel::<PeerMsg>(1000);
        
    let mut rx = ReceiverStream::new(rx);

    tokio::spawn(async move {
        while let Some(msg) = rx.next().await {
            if let Err(e) = write.send(
                Message::binary(
                    serde_json::to_vec(&msg).expect("msg does not have serde serialize trait"))
                ).await {
                    println!("{:?}", e);
                }
        }
    });

    let mut peers = context.peers.lock().expect("Lock poisoned");

    peers.set(peer_public_key, Peer{
        sender: tx,
        peer_list: Vec::new()
    });

    tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            let tx_clone = tx.clone();
            // Need validation
            let command: PeerMsg = serde_json::from_slice(&msg.into_data()).unwrap();
            tokio::spawn(async move {
                p2p_switch(
                    command, 
                    tx_clone, 
                    context.clone()
                )
            });
        }
    });

    // Add Cleanup function

    Ok(())
}

async fn p2p_switch(
    peer_msg: PeerMsg, 
    sender: Sender<PeerMsg>,
    context: Context
) {
    match peer_msg {
        Broadcast(stage) => {
            match stage {
                Init { command } => {

                },
                Echo { command } => {

                },
                Ready { hash } => {

                }
            }
        },
        PeerMsg::PeerInit { peer_list, public_key, signature } => {

        }
    }
}

async fn sign_msg(msg: &PeerMsg) -> SignedMsg {
    let SignOutput { hash, salt, signature } = self.credentials.sign(serde_json::to_string(msg).unwrap()).await.unwrap();
        SignedMsg {
            msg: msg.clone(),
            public_key: self.credentials.public_key,
            hash,
            salt,
            signature
        }
}