use std::{collections::BTreeMap, net::SocketAddr};

use derive_alias::derive_alias;
use ed25519_consensus::{Signature, SigningKey, VerificationKey};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};




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
pub struct Credentials {
    pub private_key: SigningKey,
    pub public_key: VerificationKey,
    pub nonce: u64,
}

impl Default for Credentials {
    fn default() -> Self {
        Self::new()
    }
}

impl Credentials {
    pub fn new() -> Credentials {
        let sk = SigningKey::new(thread_rng());
        let vk = VerificationKey::from(&sk);

        Self {
            private_key: sk,
            public_key: vk,
            nonce: 1,
        }
    }

    pub fn hash_sign(&self, message: &str) -> (String, Signature) {
        let private_key = self.private_key.clone();

        // Hash + Signature operation may be considered blocking

        let mut digest: Sha512 = Sha512::new();
        digest.update(message);

        let digest_string: String = format!("{:X}", digest.clone().finalize());

        let signature: Signature = private_key.sign(digest_string.as_bytes());

        (digest_string, signature)
    }

    pub fn create_client_transaction(&mut self, command: TransactionCommand) -> ClientCommand {
        // Increment nonceS
        self.nonce += 1;

        let body = TransactionBody {
            nonce: self.nonce,
            public_key: self.public_key,
            command
        };

        let message_string = serde_json::to_string(&body).unwrap();
    
        let (hash, signature) = self.hash_sign(&message_string);
    
        ClientCommand::Transaction {
            body,
            hash,
            signature,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Keys {
    Empty,
    Is(Credentials)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientCommand {
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
pub enum PeerCommand {
    TransactionBroadcast(TransactionBroadcast),
    PeerInit {
        peer_list: Vec<VerificationKey>,
        public_key: VerificationKey,
        signature: Signature,
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TransactionBroadcast {
    // Initializing a ClientCommand does not require a signature from the submitting peer
    Init {
        command: ClientCommand
    },    
    Echo {
        command: ClientCommand
    },
    Ready {
        hash: String
    }
}

#[derive(Debug)]
pub enum MsgType {
    Client(ClientCommand),
    Peer(PeerCommand)
}


pub fn socket_to_ws(addr: SocketAddr) -> String {
    let mut ws_addr = "ws://".to_string();
    ws_addr.push_str(&addr.to_string());
    return ws_addr
}

