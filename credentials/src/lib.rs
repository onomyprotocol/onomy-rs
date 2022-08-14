use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use ed25519_consensus::{Signature, SigningKey, VerificationKey};
use equity_types::MsgType;

// The HashMap

#[derive(Debug, Clone)]
    pub struct Credentials {
    sender: mpsc::Sender<Command>
}

impl Brb {
    pub fn new() -> Brb {
        let (tx, mut rx) = mpsc::channel(1000);

        tokio::spawn(async move
            {
                while let Some(cmd) = rx.recv().await {
                    match cmd {
                        Command::Sign { key, resp } => {
                            let response = brb_map.get(&key);

                            match response {
                                Some(sender) => {
                                    resp.send(Some(sender.clone())).unwrap()
                                },
                                None => resp.send(None).unwrap()
                            };
                        }
                    }
                }
            }
        );
        Brb {
            sender: tx
        }
    }

    async fn get(&self, hash: &String) -> Option<mpsc::Sender<BrbMsg>> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(BrbCommand::Get { key: hash.clone(), resp: tx }).await.unwrap();
        rx.await.unwrap()
    }

    // All BRB broadcast messages are either initiated or received.
    // Initiation is prompted by out of network messages (client / new validator) or enabled consensus condition.
    // All in-network messages are Received as part of Brb Broadcast
    pub async fn initiate (&self, hash: String, peer: VerificationKey, msg: MsgType) {
        // First need to check if there is already an initiated BRB instance with this same hash
        if let Some(brb_sender) = self.get(&hash).await {
            // BRB manager exists then treat as Echo
            // Need to define below how to use Echo before completing this part.
            brb_sender.send(BrbMsg::Echo{
                hash: hash.clone(),
                peer,
                msg
            }).await.unwrap()
        }
        
        let (brb_tx, mut brb_rx) = mpsc::channel(1000);
        let (brb_one_tx, brb_one_rx) = oneshot::channel();
        
        tokio::spawn(async move
            {
                while let Some(brb_msg) = brb_rx.recv().await {
                    match brb_msg {
                        BrbMsg::Init { hash, peer, msg } => {
                            
                        }
                        BrbMsg::Echo { hash, peer, msg } => {
                            
                        }
                        BrbMsg::Ready { hash } => {
                            
                        }
                    }
                }

                brb_one_tx.send(true).unwrap();
            }
        );

        let (one_tx, one_rx) = oneshot::channel();

        self.sender.send(BrbCommand::Set
            {
                key: hash, 
                val: brb_tx, 
                resp: one_tx
            }
        ).await.unwrap();

        match one_rx.await {
            Ok(v) => println!("got = {:?}", v),
            Err(_) => println!("the sender dropped"),
        }

        match brb_one_rx.await {
            Ok(v) => println!("got = {:?}", v),
            Err(_) => println!("the sender dropped"),
        }
    }

    pub fn receive () {

    }

}

/// Multiple different commands are multiplexed over a single channel.
/// Each Byzantine Reliable Broadcast instance has its own task that maintains state
/// The Routing HashMap stores the Senders to the Task mangaging the instance of BRB
#[derive(Debug)]
enum Command {
    Sign {
        key: String,
        resp: Responder<Option<mpsc::Sender<BrbMsg>>>,
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
        let sk = SigningKey::new(thread_rng());
        let vk = VerificationKey::from(&sk);

        Self {
            private_key: sk,
            public_key: vk,
            nonce: 1,
        }
    }
}

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<T>;
