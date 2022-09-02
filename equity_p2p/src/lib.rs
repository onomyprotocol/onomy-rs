use std::{
    collections::HashMap,
    sync::{ Mutex, Arc }
};


use futures::future::join_all;


use tokio::sync::mpsc::Sender;

use ed25519_consensus::VerificationKey;

use serde_plain;

use std::{ net::SocketAddr };

use credentials::Credentials;
use equity_types::{ EquityError, PeerMsg, Broadcast };
use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Sender},
    task::JoinHandle,
};

use equity_p2p::Peer;

use serde_json::Value;


use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;
use crate::service::Context;
use crate::error::Error;


// Phase 1: Peers will be added by direct API Request to validator
// Phase 2: Peer added as transaction as direct API access to validator not available

// Phase 1: Peer sends API request to one or more validators to join validator network
//          Peer does not initiate connections to validators. Validators initiate.
// Init:    Validators that receive validator requests send out Init messages.
//          Init messages with replicate hashes are recognized as echos.
// Echo:    


#[derive(Debug, Clone)]
pub struct Peer {
    pub sender: Sender<PeerMsg>,
    pub peer_list: Vec<VerificationKey>,
}

#[derive(Debug, Clone)]
pub struct P2P{
    pub peer_map: Arc<Mutex<HashMap<String, Peer>>,
}

impl P2P {
    pub async fn init(
        p2p_listener: SocketAddr
    ) -> Result<(SocketAddr, JoinHandle<Result<(), EquityError>>), Error> {
        let peer_map = Arc::new(Mutex::new(HashMap::new()));
        
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
        Ok((bound_addr, handle))
    }

    pub fn get(&self, key: &VerificationKey) -> Peer {
        self
            .data
            .lock()
            .expect("Lock is poisoned")
            .get(&serde_plain::to_string(key)
            .unwrap())
            .cloned()
            .unwrap()
            
    }

    pub fn set(&self, key: &VerificationKey, peer: &Peer) -> Option<Peer> {
        self
            .data
            .lock()
            .expect("Lock is poisoned")
            .insert(
                serde_plain::to_string(key).unwrap(),
                peer.clone()
            )
    }

    
    pub async fn broadcast(&self, msg: Broadcast) {
        let senders = self.senders();

        let send = senders.iter().map(|sender| sender.send(PeerMsg::Broadcast(msg.clone())));

        join_all(send).await;
    }

    pub fn senders(&self) -> Vec<Sender<PeerMsg>> {
        let peer_map = self.data
        .lock()
        .expect("Lock poisoned");

        peer_map.clone().into_values()
        .map(|peer| peer.sender)
        .collect::<Vec<Sender<PeerMsg>>>()
    }
}