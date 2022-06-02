use equity_storage::EquityDatabase;
use tokio::task::JoinHandle;

use crate::api_server::{start_api_server, EquityError};

pub struct EquityService {
    pub address: std::net::SocketAddr,
    tasks: Vec<JoinHandle<Result<(), EquityError>>>,
}

impl EquityService {
    pub async fn new(db: EquityDatabase) -> Result<Self, std::io::Error> {
        let (address, server_handle) = start_api_server(db).await?;

        let tasks = vec![server_handle];

        Ok(Self { address, tasks })
    }

    pub async fn run(self) {
        for task in self.tasks {
            let _ = task.await;
        }
    }
}
