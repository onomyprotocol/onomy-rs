use std::{
    collections::HashMap,
    sync::{Arc, Mutex}
};

use tokio::sync::mpsc::Sender;
use tungstenite::Message;
use ed25519_consensus::VerificationKey;


// Phase 1: Peers will be added by direct API Request to validator
// Phase 2: Peer added as transaction as direct API access to validator not available

// Phase 1: Peer sends API request to one or more validators to join validator network
//          Peer does not initiate connections to validators. Validators initiate.
// Init:    Validators that receive validator requests send out Init messages.
//          Init messages with replicate hashes are recognized as echos.
// Echo:    


#[derive(Debug)]
pub struct Peer {
    pub sender: Sender<Message>,
    pub peer_list: Vec<VerificationKey>,
}

pub type PeerMap = Arc<Mutex<HashMap<String, Peer>>>;
