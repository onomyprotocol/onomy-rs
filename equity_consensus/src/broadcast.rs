use std::collections::HashMap;
use std::collections::HashSet;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use ed25519_consensus::{Signature, VerificationKey};
use equity_types::{ BroadcastMsg, Broadcast, key_to_string };
use equity_p2p::PeerMap;


// The HashMap

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
        Self {
            sender: tx
        }
    }

    async fn get(&self, hash: &String) -> Option<mpsc::Sender<BrbMsg>> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(BrbCommand::Get { key: hash.clone(), resp: tx }).await.unwrap();
        rx.await.unwrap()
    }

    async fn set(&self, hash: &String, sender: mpsc::Sender<BrbMsg>) -> bool {
        let (tx, rx) = oneshot::channel();
        self.sender.send(BrbCommand::Set { key: hash.clone(), val: sender, resp: tx }).await.unwrap();
        rx.await.unwrap()
    }

    // All BRB broadcast messages are either initiated or received.
    // Initiation is prompted by out of network messages (client / new validator) or enabled consensus condition.
    // All in-network messages are Received as part of Brb Broadcast
    pub async fn initiate (&self, peers: PeerMap, hash: &String, public_key: &VerificationKey, broadcast_msg: &BroadcastMsg) {
        // First need to check if there is already an initiated BRB instance with this same hash
        if let Some(brb_sender) = self.get(&hash).await {
            // BRB manager exists then treat as Echo
            // Need to define below how to use Echo before completing this part.
            brb_sender.send(
                BrbMsg::Echo{
                    public_key: public_key.clone(),
                    broadcast_msg: broadcast_msg.clone()
                }
            ).await.unwrap()
        }
        
        let (brb_tx, mut brb_rx) = mpsc::channel(1000);
        let (brb_one_tx, brb_one_rx) = oneshot::channel();
        let hash_spawn = hash.clone();
        let broadcast_msg = broadcast_msg.clone();
        let self_internal = self.clone();

        tokio::spawn(async move
            {
                let mut internal = BrbInternal {
                    hash: hash_spawn.clone(),
                    ctl: "Echo".to_string(),
                    msg: broadcast_msg,
                    init: true,
                    echo: HashSet::new(),
                    ready: HashSet::new(),
                    timeout: HashSet::new(),
                    commit: false
                };

                while let Some(brb_msg) = brb_rx.recv().await {
                    match brb_msg {
                        BrbMsg::Init { public_key, broadcast_msg } => {
                            // First need to check if there is already a timeout
                            // Timeout caused by receiving Echo before Init msg
                            if internal.ctl == "Timeout".to_string() {
                                return
                            }
                            // If Init did not initiate the BRB then it is a timeout.  Within the strict model
                            // of communication required by this protocol.  Is that too strict?

                            // Check if this broadcast has already been initialized
                            // If broadcast has been initialized, then treat init as echo
                            if let Some(_brb_sender) = self_internal.get(&hash_spawn.clone()).await {
                                // Add public key to hashset
                                // This results in bool, do we want to punish for duplicate?
                                let _nonexists = internal.echo.insert(key_to_string(&public_key).unwrap());
                            }

                        }
                        BrbMsg::Echo { public_key, broadcast_msg } => {
                            if let Some(_brb_sender) = self_internal.get(&hash_spawn.clone()).await {
                                // Add public key to hashset
                                // This results in bool, do we want to punish for duplicate echo?
                                let _nonexists = internal.echo.insert(key_to_string(&public_key).unwrap());

                                if internal.ctl == "Echo".to_string() {
                                    // If BRB in ctl = "Echo" and cardinality of internal.echo = (n+t)/2 broadcast 
                                    // Then broadcast "Ready"
                                    
                                    // Cardinality of peers is going to get contentious - need to create manager
                                    if internal.echo.len() > peers.cardinality()/2 {
                                        // Broadcast Ready
                                        peers.broadcast(
                                        Broadcast::Ready {
                                            hash: hash_spawn.clone()
                                        }).await;
                                        
                                        internal.ctl = "Ready".to_string();
                                    }
                                }

                                if internal.ctl == "Ready".to_string() {
                                    // Step 2 (Ready) Bracha BRB: cardinality of internal.
                                }


                            } else {
                                // If BRB instance does not exist for msg.hash then Timeout

                                // Add public key to echo hashset regardless of ctl
                                // This results in bool, do we want to punish for duplicate echo?
                                let _nonexists = internal.echo.insert(key_to_string(&public_key).unwrap());

                                if internal.ctl == "Timeout" {
                                    
                                    internal.ctl = "Timeout".to_string();

                                    // BRB manager does not exist send timeout
                                    // Broadcast timeout
                                    peers.broadcast(
                                        Broadcast::Timeout {
                                            hash: hash_spawn.clone()
                                    }).await;
                                }
                            }
                        }
                        BrbMsg::Ready { hash } => {
                            
                        }
                        BrbMsg::Timeout { hash } => {

                        }
                    }
                }

                brb_one_tx.send(true).unwrap();
            }
        );

        match self.set(&hash, brb_tx).await {
            true => println!("Initiated BRB {}", &hash),
            false => println!("Failed to initiate BRB {}", &hash)
        };

        match brb_one_rx.await {
            Ok(v) => println!("got = {:?}", v),
            Err(_) => println!("the sender dropped"),
        }
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

#[derive(Debug)]
enum BrbMsg {
    Init {
        public_key: VerificationKey,
        broadcast_msg: BroadcastMsg
    },
    Echo {
        public_key: VerificationKey,
        broadcast_msg: BroadcastMsg
    },
    Ready {
        hash: String
    },
    Timeout {
        hash: String
    }
}

#[derive(Debug)]
pub struct BrbInternal {
    hash: String,
    msg: BroadcastMsg,
    ctl: String,
    init: bool,
    echo: HashSet<String>,
    ready: HashSet<String>,
    timeout: HashSet<String>,
    commit: bool
}

impl BrbInternal {
    fn init (hash: String, BroadcastMsg: BroadcastMsg, signature: Signature) {

    } 
}

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<T>;
