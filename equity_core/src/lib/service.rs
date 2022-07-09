use std::net::SocketAddr;

use equity_storage::EquityDatabase;
use tokio::task::JoinHandle;

use futures::future::join_all;
use ed25519_consensus::{SigningKey, VerificationKey};
use rand::{Rng, thread_rng};

use equity_types::EquityError;


use crate::{
    api_server::{start_api_server},
    Error,
};

pub struct EquityService {
    pub address: std::net::SocketAddr,
    tasks: Vec<JoinHandle<Result<(), EquityError>>>,
    private_key: SigningKey,
    public_key: VerificationKey
}

impl EquityService {
    pub async fn new(listener: SocketAddr, db: EquityDatabase) -> Result<Self, Error> {
        let (address, server_handle) = start_api_server(listener, db).await?;

        let tasks = vec![server_handle];

        let sk = SigningKey::new(thread_rng());
        let vk = VerificationKey::from(&sk);

        Ok(Self { 
            address: address, 
            tasks: tasks,  
            private_key: sk,
            public_key: vk
        })
    }

    pub async fn run(self) {
        join_all(self.tasks).await;
    }
}
