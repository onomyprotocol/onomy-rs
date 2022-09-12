use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use ed25519_consensus::VerificationKey;
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

    async fn exists(&self, hash: &String) -> bool {
        if let Some(brb_sender) = self.get(hash).await {
            return true
        }
        false
    }

    async fn echo(&self, internal: &mut BrbInternal, public_key: &VerificationKey, peers: &PeerMap) {
        if let Some(tally_len) = internal.update_tally(&"Echo".into(), &public_key) {
            if internal.ctl == "Echo".to_string() {
                // If BRB in ctl = "Echo" and cardinality of internal.echo = (n+t)/2 broadcast 
                // Then broadcast "Ready"
                
                // Cardinality of peers is going to get contentious - need to create manager
                if tally_len > peers.cardinality()/2 {
                    // Broadcast Ready
                    peers.broadcast(
                    Broadcast::Ready {
                        hash: internal.hash.clone()
                    }).await;
                    
                    internal.ctl = "Ready".to_string();
                }
            }
        }
    }

    // All broadcasts are initiated whether originating from self or external
    pub async fn initiate (&self, peers: PeerMap, hash: &String, public_key: &VerificationKey, broadcast_msg: &BroadcastMsg) {
        let (brb_tx, mut brb_rx) = mpsc::channel(1000);
        let (brb_one_tx, brb_one_rx) = oneshot::channel();
        let hash_spawn = hash.clone();
        let broadcast_msg = broadcast_msg.clone();
        let self_internal = self.clone();

        let internal = Arc::new(Mutex::new(
            BrbInternal {
                hash: hash_spawn,
                ctl: "Init".to_string(),
                msg: broadcast_msg,
                init: false,
                tally: HashMap::new(),
                commit: false
        }));

        tokio::spawn(async move
            {
                while let Some(brb_msg) = brb_rx.recv().await {
                    let internal_handler = internal.lock().unwrap();
                    match brb_msg {
                        BrbMsg::Init { public_key, broadcast_msg } => {
                            match internal_handler.ctl.as_str() {
                                "Init" => {

                                }

                                "Echo" => {
                                    let tally_len = internal_handler.update_tally(&"Echo".into(), &public_key).unwrap();
                                    if tally_len > peers.cardinality()/2 {
                                        let hash = internal_handler.hash.clone();
                                        // Broadcast Ready
                                        tokio::spawn( async move {
                                            peers.broadcast(
                                                Broadcast::Ready {
                                                    hash
                                                }).await;
                                        });
                                        internal_handler.ctl = "Ready".to_string();
                                    }
                                }

                                "Ready" => {

                                }
                            }

                            if internal_handler.ctl == "Echo".to_string() {
                                self_internal.echo(&mut internal_handler, &public_key, &peers).await;
                            }   
                        }
                        BrbMsg::Echo { public_key, broadcast_msg } => {
                            // Receiving echo before init, Timeout
                            // Assumes network routing through another peer is always
                            // slower than direct P2P connection.  Not sure if this
                            // holds
                            if internal_handler.ctl == "Init".to_string() {
                                internal_handler.ctl = "Timeout".to_string();

                                // Broadcast timeout
                                peers.broadcast(
                                    Broadcast::Timeout {
                                        hash: hash_spawn.clone()
                                }).await;
                            }

                            if internal_handler.ctl == "Ready".to_string() {
                                // Step 2 (Ready) Bracha BRB: cardinality of internal.
                            }
                        }
                        BrbMsg::Ready { hash } => {
                            
                        }
                        BrbMsg::Timeout { hash } => {

                        }
                    }  
                }
                brb_one_tx.send(true).unwrap();
            });

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

#[derive(Debug, Clone)]
pub struct BrbInternal {
    hash: String,
    msg: BroadcastMsg,
    ctl: String,
    init: bool,
    tally: HashMap<String, HashSet<String>>,
    commit: bool
}

impl BrbInternal {
    fn update_tally(&self, stage: &String, public_key: &VerificationKey) -> Option<usize> {
        if let Some(stage_set) = self.tally.get(stage) {
            stage_set.insert(key_to_string(public_key).unwrap());
            return Some(stage_set.len())
        }
        None
    }
}

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<T>;
