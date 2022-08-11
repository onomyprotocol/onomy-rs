use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use ed25519_consensus::{Signature, VerificationKey};

// The HashMa

#[derive(Debug, Clone)]
    pub struct Brb {
    sender: mpsc::Sender<BrbCommand>
}

impl Brb {
    pub fn new() -> Brb {
        let (tx, mut rx) = mpsc::channel(1000);

        tokio::spawn(async move
            {
                // Optimize to HashMap that uses the SHA512 directly without hashing a key
                // String and std::collections::hashmap is just a convenience for now.
                let mut brb_map: HashMap<String, mpsc::Sender<BrbMsg>> = HashMap::new();

                while let Some(cmd) = rx.recv().await {
                    match cmd {
                        BrbCommand::Get { key, resp } => {
                            let response = brb_map.get(&key);

                            match response {
                                Some(sender) => {
                                    resp.send(Some(sender.clone())).unwrap()
                                },
                                None => resp.send(None).unwrap()
                            };
                        }
                        BrbCommand::Set { key, val, resp } => {
                            let mut exists: bool = false;
                            
                            if let Some(_res) = brb_map.insert(key, val) {
                                exists = true;
                            }
                            
                            let _ = resp.send(exists);
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
    pub async fn initiate (&self, hash: String) {
        // First need to check if there is already an initiated BRB instance with this same hash
        if let Some(brb_sender) = self.get(&hash).await {
            // BRB manager exists then treat as Echo
            // Need to define below how to use Echo before completing this part.
            brb.sender.send(BrbMsg::Echo{
                hash: String,

            })
        }
        
        let (brb_tx, mut brb_rx) = mpsc::channel(1000);
        let (brb_one_tx, brb_one_rx) = oneshot::channel();
        
        tokio::spawn(async move
            {
                while let Some(brb_msg) = brb_rx.recv().await {
                    match brb_msg {
                        BrbMsg::Init { hash, command } => {
                            
                        }
                        BrbMsg::Echo { } => {
                            
                        }
                        BrbMsg::Ready { } => {
                            
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
enum BrbCommand {
    Get {
        key: String,
        resp: Responder<Option<mpsc::Sender<BrbMsg>>>,
    },
    Set {
        key: String,
        val: mpsc::Sender<BrbMsg>,
        resp: Responder<bool>,
    },
}

enum BrbMsg {
    Init {
        hash: String,
        command: ClientCommand
    },
    Echo {
        hash: String,
        peer: VerificationKey
    },
    Ready {

    }
}

enum Command {
    ClientCommand,
    PeerCommand
}

pub struct BrbInternal {
    hash: String,
    body: BrbBody,
    signature: Signature,
    init: bool,
    echo: Vec<VerificationKey>,
    ready: Vec<VerificationKey>,
    commit: bool
}

enum BrbBody {

}

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<T>;
