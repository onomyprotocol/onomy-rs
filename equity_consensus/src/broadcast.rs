use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

// The HashMa
pub struct BrbMap {
    sender: mpsc::Sender<Command>
}

impl BrbMap {
    pub fn new() {
        let (tx, rx) = mpsc::channel(1000);
        let manager = tokio::spawn(async move
            {
                let brb_map: HashMap<String, mpsc::Sender<BrbMsg>> = HashMap::new();

                while let Some(cmd) = rx.recv().await {
                    match cmd {
                        Command::Get { key, resp } => {
                            let sender = Option<mpsc::Sender<BrbMsg>>
                            if let Some(res) = brb_map.get(&key) {
                                
                            }
                            let sender = res.clone();
                            // Ignore errors
                            let _ = resp.send(sender);
                        }
                        Command::Set { key, val, resp } => {
                            let res = brb_map.insert(key, val);
                            // Ignore errors
                            let _ = resp.send(res);
                        }
                    }
                }
            }
            
        );
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
        resp: Responder<()>,
    },
}

enum BrbSender {
    Some(),
    None
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
