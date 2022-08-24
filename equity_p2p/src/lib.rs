use std::{
    collections::HashMap,
    sync::{ Mutex, Arc }
};

use tokio::sync::mpsc::Sender;
use equity_types::PeerMsg;
use ed25519_consensus::VerificationKey;

use serde_plain;


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
pub struct PeerMap{
    pub data: Arc<Mutex<HashMap<String, Peer>>>
}

impl PeerMap {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
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
}