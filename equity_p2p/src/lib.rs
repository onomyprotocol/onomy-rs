use std::{
    collections::{HashMap, HashMap::get}
    sync::{Arc, Mutex}
};

use tokio::sync::mpsc::Sender;
use tungstenite::Message;
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
    pub sender: Sender<Message>,
    pub peer_list: Vec<VerificationKey>,
}

#[derive(Debug)]
pub struct PeerMap{
    data_holder: Mutex<HashMap<String, Peer>>
};

impl PeerMap {
    fn get(&self, key: &VerificationKey) -> Peer {
        self
            .data_holder
            .lock()
            .expect("Lock is poisoned")
            .get(&serde_plain::to_string(key)
            .unwrap())
            .cloned()
            .unwrap()
            
    }

    fn set(&self, key: &VerificationKey, peer: &Peer) -> Option<Peer> {
        self
            .data_holder
            .lock()
            .expect("Lock is poisoned")
            .insert(
                serde_plain::to_string(key).unwrap(),
                peer.clone()
            )
    }
}