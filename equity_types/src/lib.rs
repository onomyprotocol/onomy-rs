use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};

pub use borsh;
use borsh::{BorshDeserialize, BorshSerialize};
use derive_alias::derive_alias;
use ed25519_consensus::{Signature, SigningKey, VerificationKey};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

// TODO common derive macro

derive_alias! {
    derive_common => #[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord,
        BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
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
}
