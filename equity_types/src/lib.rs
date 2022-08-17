use std::{collections::BTreeMap, net::SocketAddr};
use derive_alias::derive_alias;
use ed25519_consensus::{Signature, VerificationKey};
use serde::{Deserialize, Serialize};

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
pub struct Value(pub u64);
}

derive_common! {
pub struct EquityAddressResponse {
    pub owner: String,
    pub value: Value,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FullMessage {
    pub body: Body,
    pub hash: String,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Body {
    pub public_key: VerificationKey,
    pub nonce: u64,
    pub keys_values: BTreeMap<u64, u64>,
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
    Transaction {
        body: TransactionBody,
        hash: String,
        signature: Signature,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionBody {
    nonce: u64,
    public_key: VerificationKey,
    command: TransactionCommand
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
pub enum PeerMsg {
    PeerInit {
        peer_list: Vec<VerificationKey>,
        public_key: VerificationKey,
        signature: Signature,
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Broadcast {
    Init {
        msg: Msg,
        public_key: VerificationKey,
        signature: Signature
    },    
    Echo {
        msg: Msg
    },
    Ready {
        hash: String
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Msg {
    Client(ClientMsg),
    Peer(PeerMsg)
}


pub fn socket_to_ws(addr: SocketAddr) -> String {
    let mut ws_addr = "ws://".to_string();
    ws_addr.push_str(&addr.to_string());
    return ws_addr
}

