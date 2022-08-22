use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use ed25519_consensus::{Signature, SigningKey, VerificationKey};
use sha2::{Digest, Sha512};
use rand::thread_rng;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct KeyPair {
    private_key: SigningKey,
    public_key: VerificationKey
}

#[derive(Debug, Clone)]
pub enum Keys {
    Empty,
    Is(KeyPair)
}

#[derive(Debug, Clone)]
pub struct Credentials {
    public_key: VerificationKey,
    sender: mpsc::Sender<Command>
}

impl Credentials {
    pub fn new(keys: Keys) -> Credentials {
        let (tx, mut rx) = mpsc::channel(1000);

        let signer = Internal::new(keys);

        let signer_spawn = signer.clone();

        tokio::spawn(async move
            {
                while let Some(cmd) = rx.recv().await {
                    
                    match cmd {
                        Command::Sign { input, resp } => {
                            
                            let (hash, signature) = signer_spawn.sign(input);

                            resp.send(
                                Some(
                                    Response::Sign { 
                                        hash, 
                                        signature
                                    }
                                )
                            ).unwrap()
                        }
                    }
                }
            }
        );

        Self {
            public_key: signer.public_key,
            sender: tx
        }
    }

    pub async fn sign<T: Clone, Deserialize>(&self, input: String) -> Option<SignedMsg> {
        let (resp, rx) = oneshot::channel();
        self.sender.send(Command::Sign { input: input.clone(), resp }).await.unwrap();
        if let Some(Response::Sign{signed_msg}) = rx.await.unwrap() {
            Some(signed_msg)
        } else {
            None
        }
    }
}

/// Multiple different commands are multiplexed over a single channel.
/// Each Byzantine Reliable Broadcast instance has its own task that maintains state
/// The Routing HashMap stores the Senders to the Task mangaging the instance of BRB
#[derive(Debug)]
enum Command {
    Sign {
        input: String,
        resp: Responder<Option<Response>>,
    }
}

#[derive(Debug)]
enum Response {
    Sign {
        hash: String,
        nonce: u64,
        signature: Signature
    }
}

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<T>;

#[derive(Debug, Clone)]
struct Internal {
    pub private_key: SigningKey,
    pub public_key: VerificationKey,
    pub nonce: u64,
}

impl Internal {
    fn new(keys: Keys) -> Internal {

        match keys {
            Keys::Empty => {
                let sk = SigningKey::new(thread_rng());
                let vk = VerificationKey::from(&sk);

                Self {
                    private_key: sk,
                    public_key: vk,
                    nonce: 1,
                }
            },
            Keys::Is(cred) => {
                Self {
                    private_key: cred.private_key,
                    public_key: cred.public_key,
                    nonce: 1
                }
            },
        }
        
    }

    fn sign(&self, input: String) -> (String, Signature) {
        let salt: u64 = thread_rng().gen::<u64>();

        // Hash + Signature operation may be considered blocking
        let mut digest: Sha512 = Sha512::new();
        
        digest.update(message);

        let hash: String = format!("{:X}", digest.clone().finalize());

        let signature: Signature = self.private_key.sign(hash.as_bytes());

        (hash, signature)
    }
}


