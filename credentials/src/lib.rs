use equity_types::SignOutput;
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
    private_key: SigningKey,
    public_key: VerificationKey,
    sender: mpsc::Sender<Command>
}

// May not even need the internal structure anymore because using salt instead of a nonce

impl Credentials {
    pub fn new(keys: Keys) -> Credentials {
        let (tx, mut rx) = mpsc::channel(1000);

        match keys {
            Keys::Empty => {
                let sk = SigningKey::new(thread_rng());
                let vk = VerificationKey::from(&sk);

                Self {
                    private_key: sk,
                    public_key: vk,
                    sender: tx
                };
            },
            Keys::Is(cred) => {
                Self {
                    private_key: cred.private_key,
                    public_key: cred.public_key,
                    sender: tx
                };
            },
        }

        

        tokio::spawn(async move
            {
                while let Some(cmd) = rx.recv().await {
                    match cmd {
                        Command::Sign { input, resp } => {
                            tokio::spawn(async move {
                                let salt: u64 = thread_rng().gen::<u64>();

                                // Hash + Signature operation may be considered blocking
                                let mut digest: Sha512 = Sha512::new();
                                
                                digest.update(input);

                                let hash: String = format!("{:X}", digest.finalize());

                                let signature: Signature = self.private_key.sign(hash.as_bytes());

                                let response = Response::Sign {
                                    hash,
                                    salt,
                                    signature
                                };

                                resp.send(
                                    Some(response)
                                ).unwrap()
                            }).await.unwrap()
                        }
                    }
                }
            }
        );
    }

    pub async fn sign(&self, input: String) -> Option<SignOutput> {
        let (resp, rx) = oneshot::channel();
        let signed_output = self.sender.send(Command::Sign { input, resp }).await.unwrap();
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
        salt: u64,
        signature: Signature
    }
}

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<T>;
