use std::collections::HashMap;
use tokio::sync::mpsc;

use tokio::sync::oneshot;
use bytes::Bytes;

/// Multiple different commands are multiplexed over a single channel.
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<mpsc::Sender<BrbMsg>>>,
    },
    Set {
        key: String,
        val: Bytes,
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
