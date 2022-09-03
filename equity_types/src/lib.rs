use std::{collections::BTreeMap, net::SocketAddr};
use derive_alias::derive_alias;
use ed25519_consensus::{Signature, VerificationKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// TODO common derive macro

derive_alias! {
    derive_common => #[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord,
        Serialize, Deserialize)]
}

derive_common! {
pub struct EquityTx {
    pub from: String,
    pub to: String,
    pub amount: u64,
}
}

derive_common! {
pub struct IntValue(pub u64);
}

derive_common! {
pub struct EquityAddressResponse {
    pub owner: String,
    pub value: IntValue,
}
}

derive_common! {
pub struct PostTransactionResponse {
    pub success: bool,
    pub msg: String,
}
}

derive_common! {
pub struct HealthResponse {
    pub up: bool,
}
}

#[derive(Debug, thiserror::Error)]
pub enum EquityError {
    #[error("An api server error occurred {0}")]
    ApiServer(#[from] hyper::Error),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientMsg {
    Health {
    },
    Transaction(Transaction)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TransactionCommand {
    SetValues {
        keys_values: BTreeMap<u64, u64>,
    },
    SetValidator {
        ws: SocketAddr
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]

pub struct SignedMsg {
    pub msg: PeerMsg,
    pub public_key: VerificationKey,
    pub hash: String,
    pub salt: u64,
    pub signature: Signature
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Consensus {

}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PeerMsg {
    PeerInit {
        peer_list: Vec<VerificationKey>,
    },
    Broadcast(Broadcast)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Broadcast {
    Init {
        msg: BroadcastMsg,
        // Signed by broadcaster
        public_key: VerificationKey,
        salt: u64,
        signature: Signature
    },    
    Echo {
        msg: BroadcastMsg,
        // Signed by broadcaster
        public_key: VerificationKey,
        salt: u64,
        signature: Signature
    },
    Ready {
        hash: String
    },
    Timeout {
        hash: String
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub command: TransactionCommand,
    pub public_key: VerificationKey,
    pub hash: String,
    pub salt: u64,
    pub signature: Signature
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BroadcastMsg {
    Transaction(Transaction),
    Consensus(Consensus)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignInput {
    pub input: String,
    pub salt: u64
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignOutput {
    pub hash: String,
    pub salt: u64,
    pub signature: Signature
}


pub fn socket_to_ws(addr: &SocketAddr) -> String {
    let mut ws_addr = "ws://".to_string();
    ws_addr.push_str(&addr.to_string());
    return ws_addr
}

pub fn key_to_string(key: &VerificationKey) -> Result<String, serde_json::Error> {
    let result = serde_json::from_slice::<Value>(&key.to_bytes());
    match result {
        Ok(val) => Ok(val.to_string()),
        Err(e) => Err(e)
    }
}

