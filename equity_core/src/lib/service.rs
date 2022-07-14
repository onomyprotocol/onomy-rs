use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;


use equity_storage::EquityDatabase;
use tokio::task::JoinHandle;

use futures::future::join_all;

use equity_types::{EquityError, PeerMap, Credentials};

use crate::{
    api_server::{start_api_server},
    p2p_server::{start_p2p_server},
    Error,
};

pub struct EquityService {
    pub api_address: std::net::SocketAddr,
    pub p2p_address: std::net::SocketAddr,
    tasks: Vec<JoinHandle<Result<(), EquityError>>>
}

impl EquityService {
    pub async fn new(api_listener: SocketAddr, p2p_listener: SocketAddr, seed_address: SocketAddr, db: EquityDatabase) -> Result<Self, Error> {
        let peers = PeerMap::new(Mutex::new(HashMap::new()));
        let credentials = Arc::new(Credentials::new());

        let (api_address, api_server_handle) = start_api_server(api_listener, db.clone(), peers.clone(), credentials.clone()).await?;
        let (p2p_address, p2p_server_handle) = start_p2p_server(p2p_listener, seed_address, db.clone(), peers.clone(), credentials.clone()).await?;

        let tasks = vec![api_server_handle, p2p_server_handle];

        Ok(Self { 
            api_address: api_address,
            p2p_address: p2p_address,
            tasks: tasks
        })
    }

    pub async fn run(self) {
        join_all(self.tasks).await;
    }
}
