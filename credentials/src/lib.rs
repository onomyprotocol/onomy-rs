use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use ed25519_consensus::{Signature, SigningKey, VerificationKey};
use equity_types::MsgType;

#[derive(Debug, Clone)]
pub enum Keys {
    Empty,
    Is({
        private_key: SigningKey,
        public_key: VerificationKey
    })
}

#[derive(Debug, Clone)]
pub struct Credentials {
    public_key: VerificationKey,
    sender: mpsc::Sender<Command>
}

impl Credentials {
    pub fn new(keys: Keys) -> Credentials {
        let (tx, mut rx) = mpsc::channel(1000);

        let signer = Internal::new();

        tokio::spawn(async move
            {
                while let Some(cmd) = rx.recv().await {
                    
                    // signer = signer.clone();

                    match cmd {
                        Command::Sign { msg, resp } => {
                            
                            // Hash + Signature operation may be considered blocking

                            let mut digest: Sha512 = Sha512::new();
                            
                            digest.update(msg);

                            let digest_string: String = format!("{:X}", digest.finalize());

                            let signature = signer.private_key.sign(digest_string.as_bytes());

                            resp.send(Some((digest_string, signature))).unwrap();
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

    async fn sign(&self, msg: &String) -> Option<(String, Signature)> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(Command::Sign { msg: msg, resp: tx }).await.unwrap();
        rx.await.unwrap()
    }
}

/// Multiple different commands are multiplexed over a single channel.
/// Each Byzantine Reliable Broadcast instance has its own task that maintains state
/// The Routing HashMap stores the Senders to the Task mangaging the instance of BRB
#[derive(Debug)]
enum Command {
    Sign {
        msg: String,
        resp: Responder<Option<mpsc::Sender<Response>>>,
    }
}

#[derive(Debug)]
struct Internal {
    pub private_key: SigningKey,
    pub public_key: VerificationKey,
    pub nonce: u64,
}

impl Internal {
    fn new() -> Internal {

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

    fn sign()
}

#[derive(Debug)]
enum Response {
    Sign {
        hash: String,
        signature: Signature
    }
}

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<T>;
