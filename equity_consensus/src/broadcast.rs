use std::collections::HashMap;
use sha2::Digest;
use tokio::sync::mpsc;

use tokio::sync::oneshot;

pub struct BrbMap {
    mapping: HashMap<dyn Digest, mpsc::Sender<BrbMsg>>
}

/// Multiple different commands are multiplexed over a single channel.
/// Each Byzantine Reliable Broadcast instance has its own task that maintains state
/// The Routing HashMap stores the Senders to the Task mangaging the instance of BRB
#[derive(Debug)]
enum Command {
    Get {
        key: Digest,
        resp: Responder<Option<mpsc::Sender<BrbMsg>>>,
    },
    Set {
        key: Digest,
        val: mpsc::Sender<BrbMsg>,
        resp: Responder<()>,
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
