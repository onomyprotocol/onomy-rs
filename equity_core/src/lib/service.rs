use std::net::SocketAddr;

use equity_storage::EquityDatabase;
use tokio::task::JoinHandle;

use futures::future::join_all;

use crate::{
    api_server::{start_api_server, EquityError},
    Error,
};
pub struct EquityService {
    pub address: std::net::SocketAddr,
    tasks: Vec<JoinHandle<Result<(), EquityError>>>,
}

impl EquityService {
    pub async fn new(listener: SocketAddr, db: EquityDatabase) -> Result<Self, Error> {
        let (address, server_handle) = start_api_server(listener, db).await?;

        let tasks = vec![server_handle];

        Ok(Self { address, tasks })
    }

    pub async fn run(self) {
        join_all(self.tasks).await;
    }
}
