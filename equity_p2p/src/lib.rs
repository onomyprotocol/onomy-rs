use std::{
    collections::HashMap,
    sync::{Arc, Mutex}
};

use tokio::sync::mpsc::Sender;
use tungstenite::Message;
use ed25519_consensus::{VerificationKey};


#[derive(Debug)]
pub struct Peer {
    pub send: Sender<Message>,
    pub public_key: VerificationKey,
    pub peer_map: HashMap<String, VerificationKey>,
}

pub type PeerMap = Arc<Mutex<HashMap<String, Peer>>>;
