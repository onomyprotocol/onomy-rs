use std::net::SocketAddr;

use equity_storage::EquityDatabase;
use tokio::task::JoinHandle;

use futures::future::join_all;
use ed25519_consensus::{SigningKey, VerificationKey};
use rand::{thread_rng};

use equity_types::EquityError;


use crate::{
    api_server::{start_api_server},
    p2p_server::{start_p2p_server},
    Error,
};

pub struct EquityService {
    pub api_address: std::net::SocketAddr,
    pub p2p_address: std::net::SocketAddr,
    tasks: Vec<JoinHandle<Result<(), EquityError>>>,
    private_key: SigningKey,
    public_key: VerificationKey
}

impl EquityService {
    pub async fn new(api_listener: SocketAddr, p2p_listener: SocketAddr, db: EquityDatabase) -> Result<Self, Error> {
        let (api_address, api_server_handle) = start_api_server(api_listener, db.clone()).await?;
        let (p2p_address, p2p_server_handle) = start_p2p_server(p2p_listener, db).await?;

        let tasks = vec![api_server_handle, p2p_server_handle];

        let sk = SigningKey::new(thread_rng());
        let vk = VerificationKey::from(&sk);

        Ok(Self { 
            api_address: api_address,
            p2p_address: p2p_address,
            tasks: tasks,  
            private_key: sk,
            public_key: vk
        })
    }

    pub async fn run(self) {
        join_all(self.tasks).await;
    }
}
