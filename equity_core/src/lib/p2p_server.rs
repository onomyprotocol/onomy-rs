use std::{collections::HashMap, net::SocketAddr};

use ed25519_consensus::{VerificationKey};
use equity_types::{ Credentials, EquityError, PeerCommand, TransactionBody, TransactionBroadcast::{ Init, Echo, Ready }, socket_to_ws };
use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Sender},
    task::JoinHandle,
};

use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;
use equity_p2p::Peer;
use crate::service::Context;
use crate::error::Error;

pub async fn start_p2p_server(
    p2p_listener: SocketAddr,
    seed_address: SocketAddr,
    context: Context
) -> Result<(SocketAddr, JoinHandle<Result<(), EquityError>>), Error> {
    if seed_address.to_string() != *"0.0.0.0:0" {
        initialize_network(
            &socket_to_ws(seed_address),
            context.clone(),
            p2p_listener,
        )
        .await;
    }

    let try_socket = TcpListener::bind(&p2p_listener).await;
    let listener = try_socket.expect("Failed to bind");
    let bound_addr = listener.local_addr().unwrap();

    info!(target: "equity-core", "Starting P2P Server");

    let handle = tokio::spawn(async move {
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

    info!(target: "equity-core", "P2P Server started at: {}", bound_addr);

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

        let init_message: PeerCommand =
            serde_json::from_str(&initial_msg.into_text().unwrap()).unwrap();
    }
    
    tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            let tx_clone = tx.clone();
            // Need validation
            let command: PeerCommand = serde_json::from_slice(&msg.into_data()).unwrap();
            tokio::spawn(async move {
                p2p_switch(
                    command, 
                    tx_clone, 
                    context.clone()
                )
            });
        }
    });
    Ok(())

    let mut peers = context.peers.lock().unwrap();

    peers.remove(&listener);
}

async fn initialize_network(
    seed_address: &String,
    context: Context,
    p2p_listener: SocketAddr,
) {
    let (mut ws_stream, _) = connect_async(seed_address)
        .await
        .expect("Failed to connect");

    println!("WebSocket handshake has been successfully completed");
    
    // Send ClientCommand
    ws_stream
        .send(initial_message(&context.credentials, p2p_listener))
        .await
        .unwrap();

    let (write, mut read) = ws_stream.split();

    // Insert the write part of this peer to the peer map.
    // I do need to receive the peermap, but that will be in handle connection.
    let (tx, rx) = channel(1000);
    let rx = ReceiverStream::new(rx);

    tokio::spawn(rx.map(Ok).forward(write));

    let mut seed_peer_map: HashMap<String, VerificationKey> = HashMap::new();

    if let Some(init_resp_msg) = read.next().await {
        let init_resp_msg = init_resp_msg.unwrap();

        let init_resp_msg: InitResponse =
            serde_json::from_str(&init_resp_msg.into_text().unwrap()).unwrap();

        seed_peer_map = init_resp_msg.peer_map.clone();

        let mut peers = context.peers.lock().unwrap();

        let peer_struct = Peer {
            send: tx.clone(),
            public_key: init_resp_msg.public_key,
            peer_map: init_resp_msg.peer_map,
        };

        peers.insert(seed_address.clone(), peer_struct);
    }

    // Iterate over everything.
    for (adr, _key) in seed_peer_map {
        
    }
}

pub fn initial_message(credentials: &Credentials, p2p_listener: SocketAddr) -> Message {
    let transaction_body = TransactionBody::SetValidator {
        public_key: credentials.public_key,
        nonce: credentials.nonce,
        ws: p2p_listener
    };

    let transaction = credentials.create_transaction(&transaction_body);

    Message::binary(
        serde_json::to_vec(&transaction)
        .unwrap(),
    )
}

pub async fn peer_connection(peer_address: SocketAddr, context: &Context) -> Result<(), Error> {
    let (mut ws_stream, _) = connect_async(socket_to_ws(peer_address))
        .await
        .expect("Failed to connect");

    println!("WebSocket handshake has been successfully completed");
    
    // Send ClientCommand
    ws_stream
        .send(initial_message(&context.credentials, p2p_listener))
        .await
        .unwrap();

    let (write, mut read) = ws_stream.split();

    // Insert the write part of this peer to the peer map.
    // I do need to receive the peermap, but that will be in handle connection.
    let (tx, rx) = channel(1000);
    let rx = ReceiverStream::new(rx);

    tokio::spawn(rx.map(Ok).forward(write));

    tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            let tx_clone = tx.clone();
            // Need validation
            let command: PeerCommand = serde_json::from_slice(&msg.into_data()).unwrap();
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
    peer_command: PeerCommand, 
    sender: Sender<Message>,
    context: Context
) {
    match peer_command {
        PeerCommand::TransactionBroadcast(stage) => {
            match stage {
                Init { command } => {

                },
                Echo { command } => {

                },
                Ready { hash } => {

                }
            }
        },
        PeerCommand::PeerInit { peer_list, public_key, signature } => {

        }
    }
}