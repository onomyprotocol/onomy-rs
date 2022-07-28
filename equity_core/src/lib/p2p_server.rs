use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use ed25519_consensus::{Signature, VerificationKey};
use equity_storage::EquityDatabase;
use equity_types::{Credentials, EquityError, Peer, PeerMap};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::channel,
    task::JoinHandle,
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;
use equity_consensus::Brb;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Initiate {
    pub public_key: VerificationKey,
    pub nonce: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct InitMessage {
    pub initiate: Initiate,
    pub listener: String,
    pub hash: String,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct InitResponse {
    pub peer_map: HashMap<String, VerificationKey>,
    pub public_key: VerificationKey,
    pub hash: String,
    pub signature: Signature,
}

pub async fn start_p2p_server(
    p2p_listener: SocketAddr,
    seed_address: SocketAddr,
    _db: EquityDatabase,
    peers: PeerMap,
    _brb: Brb,
    credentials: Arc<Credentials>,
) -> Result<(SocketAddr, JoinHandle<Result<(), EquityError>>), Error> {
    if seed_address.to_string() != *"0.0.0.0:0" {
        let mut seed_address_ws = "ws://".to_string();
        seed_address_ws.push_str(&seed_address.to_string());

        let mut p2p_address_ws = "ws://".to_string();
        p2p_address_ws.push_str(&p2p_listener.to_string());

        initialize_network(
            &seed_address_ws,
            peers.clone(),
            &credentials,
            &p2p_address_ws,
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
                peers.clone(),
                credentials.clone(),
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
    peers: PeerMap,
    credentials: Arc<Credentials>,
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

    let mut listener: String = "0.0.0.0:0000".to_string();

    if let Some(initial_msg) = read.next().await {
        let initial_msg = initial_msg.unwrap();

        let init_message: InitMessage =
            serde_json::from_str(&initial_msg.into_text().unwrap()).unwrap();

        listener = init_message.listener;

        let mut peer_map: HashMap<String, VerificationKey> = HashMap::new();

        {
            let mut peers = peers.lock().unwrap();

            let peers_iter = peers.iter();

            // Iterate over everything.
            for (adr, peer) in peers_iter {
                peer_map.insert(adr.clone(), peer.public_key);
            }

            let peer_struct = Peer {
                send: tx.clone(),
                public_key: init_message.initiate.public_key,
                peer_map: peer_map.clone(),
            };

            peers.insert(listener.clone(), peer_struct);

            drop(peers);
        }

        let peer_map_string = serde_json::to_string(&peer_map).unwrap();

        let (peer_map_hash, peer_map_signature) = credentials.hash_sign(&peer_map_string);

        let init_response = InitResponse {
            peer_map,
            public_key: credentials.public_key,
            hash: peer_map_hash,
            signature: peer_map_signature,
        };

        tx.send(Message::binary(
            serde_json::to_string(&init_response).unwrap(),
        ))
        .await
        .expect("Error during send");
    }

    while let Some(msg) = read.next().await {
        println!("Received msg: {:?}", msg);
    }

    let mut peers = peers.lock().unwrap();

    peers.remove(&listener);
}

async fn initialize_network(
    seed_address: &String,
    peers: PeerMap,
    credentials: &Credentials,
    listener: &str,
) {
    let (mut ws_stream, _) = connect_async(seed_address)
        .await
        .expect("Failed to connect");

    println!("WebSocket handshake has been successfully completed");

    ws_stream
        .send(initial_message(credentials, listener))
        .await
        .unwrap();

    let (write, mut read) = ws_stream.split();

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = channel(1000);
    let rx = ReceiverStream::new(rx);

    tokio::spawn(rx.map(Ok).forward(write));

    let mut seed_peer_map: HashMap<String, VerificationKey> = HashMap::new();

    if let Some(init_resp_msg) = read.next().await {
        let init_resp_msg = init_resp_msg.unwrap();

        let init_resp_msg: InitResponse =
            serde_json::from_str(&init_resp_msg.into_text().unwrap()).unwrap();

        seed_peer_map = init_resp_msg.peer_map.clone();

        let mut peers = peers.lock().unwrap();

        let peer_struct = Peer {
            send: tx.clone(),
            public_key: init_resp_msg.public_key,
            peer_map: init_resp_msg.peer_map,
        };

        peers.insert(seed_address.clone(), peer_struct);
    }

    // Iterate over everything.
    for (adr, _key) in seed_peer_map {
        println!("Address: {}", adr);
        let (mut ws_stream, _) = connect_async(&adr).await.expect("Failed to connect");

        println!("WebSocket handshake has been successfully completed");

        ws_stream
            .send(initial_message(credentials, listener))
            .await
            .unwrap();

        let (write, mut read) = ws_stream.split();

        // Insert the write part of this peer to the peer map.
        let (tx, rx) = channel(1000);
        let rx = ReceiverStream::new(rx);

        tokio::spawn(rx.map(Ok).forward(write));

        if let Some(init_resp_msg) = read.next().await {
            let init_resp_msg = init_resp_msg.unwrap();

            let init_resp_msg: InitResponse =
                serde_json::from_str(&init_resp_msg.into_text().unwrap()).unwrap();

            // Need to verify msg against VerificationKey

            let mut peers = peers.lock().unwrap();

            let peer_struct = Peer {
                send: tx.clone(),
                public_key: init_resp_msg.public_key,
                peer_map: init_resp_msg.peer_map,
            };

            peers.insert(adr, peer_struct);
        }
    }
}

pub fn initial_message(credentials: &Credentials, listener: &str) -> Message {
    let initiate: Initiate = Initiate {
        public_key: credentials.public_key,
        nonce: credentials.nonce,
    };

    let message_string = serde_json::to_string(&initiate).unwrap();

    let (hash, signature) = credentials.hash_sign(&message_string);

    let listener = listener.to_string();

    Message::binary(
        serde_json::to_vec(&InitMessage {
            initiate,
            listener,
            hash,
            signature,
        })
        .unwrap(),
    )
}
