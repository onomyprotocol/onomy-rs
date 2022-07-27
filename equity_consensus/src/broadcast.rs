use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

// The HashMa
pub struct BrbMap {
    sender: mpsc::Sender<Command>
}

impl BrbMap {
    pub fn new() -> BrbMap {
        let (tx, mut rx) = mpsc::channel(1000);
        tokio::spawn(async move
            {
                // Optimize to HashMap that uses the SHA512 directly without hashing a key
                // String and std::collections::hashmap is just a convenience for now.
                let mut brb_map: HashMap<String, mpsc::Sender<BrbMsg>> = HashMap::new();

                while let Some(cmd) = rx.recv().await {
                    match cmd {
                        Command::Get { key, resp } => {
                            let mut sender = None;
                            if let Some(res) = brb_map.get(&key) {
                                sender = Some(res.clone());
                            }
                            let _ = resp.send(sender);
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
        BrbMap {
            sender: tx
        }
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

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<T>;
