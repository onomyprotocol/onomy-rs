use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use ed25519_consensus::{Signature, VerificationKey};

// The HashMa

#[derive(Debug, Clone)]
    pub struct Brb {
    sender: mpsc::Sender<Command>
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
                        Command::Get { key, resp } => {
                            let response = brb_map.get(&key);

                            match response {
                                Some(res) => {
                                    res = &res.clone();
                                    res.send(BrbMsg::Echo{});
                                    resp.send(Some(res))
                                },
                                None => resp.send(None)
                            };
                        }
                        Command::Set { key, val, resp } => {
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

    async fn get(hash: String) -> Option<mpsc::Sender<BrbMsg>> {

    }

    // All BRB broadcast messages are either initiated or received.
    // Initiation is prompted by out of network messages (client / new validator) or enabled consensus condition.
    // All in-network messages are Received as part of Brb Broadcast
    pub async fn initiate (&self, msg: hash: String) {
        // First need to check if there is already an initiated BRB instance with this same hash
        


        if let Some() = self.sender.send(Command::Get{

        })
        
        let (brb_tx, mut brb_rx) = mpsc::channel(1000);
        let (brb_one_tx, brb_one_rx) = oneshot::channel();
        
        tokio::spawn(async move
            {
                while let Some(brb_msg) = brb_rx.recv().await {
                    match brb_msg {
                        BrbMsg::Init { } => {
                            
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

        self.sender.send(Command::Set
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

    },
    Echo {

    },
    Ready {

    }
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
