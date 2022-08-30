use equity_types::{ SignInput, SignOutput, BroadcastMsg, Transaction};
use tokio::sync::oneshot;
use ed25519_consensus::{Signature, SigningKey, VerificationKey};
use sha2::{Digest, Sha512};
use rand::thread_rng;
use rand::Rng;

#[derive(Debug, Clone)]
pub enum Keys {
    Empty,
    Is(Credentials)
}

#[derive(Debug, Clone)]
pub struct Credentials {
    private_key: SigningKey,
    pub public_key: VerificationKey
}

// May not even need the internal structure anymore because using salt instead of a nonce

impl Credentials {
    pub fn new(keys: Keys) -> Credentials {
        match keys {
            Keys::Empty => {
                let sk = SigningKey::new(thread_rng());
                let vk = VerificationKey::from(&sk);

                Self {
                    private_key: sk,
                    public_key: vk
                }
            },
            Keys::Is(cred) => {
                Self {
                    private_key: cred.private_key,
                    public_key: cred.public_key
                }
            },
        }
    }

    pub async fn sign(&self, input: String) -> Option<SignOutput> {
        let (resp, rx) = oneshot::channel();
        let signer = self.private_key.clone();
        
        tokio::spawn(async move {
            let salt: u64 = thread_rng().gen::<u64>();

            let mut digest: Sha512 = Sha512::new();

            digest.update(serde_json::to_string(&SignInput{
                input,
                salt
            }).unwrap());

            let hash: String = format!("{:X}", digest.finalize());

            let signature: Signature = signer.sign(hash.as_bytes());

            let response = Response::Sign {
                hash,
                salt,
                signature
            };

            resp.send(
                Some(response)
            ).unwrap()
        }).await.unwrap();
        
        if let Some(Response::Sign{ hash, salt, signature }) = rx.await.unwrap() {
            Some(SignOutput {
                hash,
                salt,
                signature
            })
        } else {
            None
        }
    }

    pub fn verify_broadcaster(msg: &BroadcastMsg, public_key: &VerificationKey, salt: &u64, signature: &Signature) -> bool {
        let mut digest: Sha512 = Sha512::new();
    
        digest.update(serde_json::to_string(&SignInput{
            input: serde_json::to_string(msg).unwrap(),
            salt: salt.clone()
        }).unwrap());
    
        let hash: String = format!("{:X}", digest.finalize());

        if let Ok(()) = public_key.verify(signature, &hash.as_bytes()) {
            true
        } else { false }
    }

    pub fn verify_signature(transaction: &Transaction) -> bool {
        let mut digest: Sha512 = Sha512::new();
    
        digest.update(serde_json::to_string(&SignInput{
            input: serde_json::to_string(&transaction.command).unwrap(),
            salt: transaction.salt
        }).unwrap());
    
        let hash: String = format!("{:X}", digest.finalize());
    
        if let Ok(()) = transaction.public_key.verify(&transaction.signature, &hash.as_bytes()) {
            true
        } else { false }
    }
}

#[derive(Debug)]
enum Response {
    Sign {
        hash: String,
        salt: u64,
        signature: Signature
    }
}



