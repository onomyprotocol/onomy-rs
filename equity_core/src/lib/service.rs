use std::net::SocketAddr;
    

use equity_storage::EquityDatabase;
use equity_consensus::Brb;
use equity_types::{ EquityError, socket_to_ws, TransactionCommand };
use futures::future::join_all;
use tokio::task::JoinHandle;
use equity_p2p::PeerMap;
use equity_client::EquityClient;
use credentials::{ Keys, Credentials };
use ed25519_consensus::VerificationKey;

use crate::{client_server::start_client_server, p2p_server::start_p2p_server, Error};

pub struct EquityService {
    pub api_address: std::net::SocketAddr,
    pub p2p_address: std::net::SocketAddr,
    tasks: Vec<JoinHandle<Result<(), EquityError>>>,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub peers: PeerMap,
    pub db: EquityDatabase,
    pub credentials: Credentials,
    pub brb: Brb
}

impl EquityService {
    pub async fn new(
        api_listener: SocketAddr,
        p2p_listener: SocketAddr,
        seed_address: SocketAddr,
        seed_public_key: Option<VerificationKey>,
        db: EquityDatabase,
    ) -> Result<Self, Error> {

        let credentials = Credentials::new(Keys::Empty);

        // Need to add in command line or file based input of keys
        let keys = Keys::Is(credentials.clone());
        
        let peers = PeerMap::new();

        let brb = Brb::new();

        let context = Context {
            peers,
            db,
            credentials,
            brb
        };

        let (api_address, api_server_handle) =
            start_client_server(api_listener, context.clone()).await?;

        // Needs to connect to Localhost if there is no other seed
        let client = EquityClient::new(&socket_to_ws(&seed_address), keys).await.unwrap();

        let (p2p_address, p2p_server_handle) = start_p2p_server(
            p2p_listener,
            context
        )
        .await?;

        // Send init validator TX to Seed Peer.
        if seed_address.to_string() != *"127.0.0.1:4040" {
            client.send_transaction(
                client.sign_transaction(
                    &TransactionCommand::SetValidator { ws: p2p_listener }
                ).await
            ).await;
        }

        let tasks = vec![api_server_handle, p2p_server_handle];

        Ok(Self {
            api_address,
            p2p_address,
            tasks,
        })
    }

    pub async fn run(self) {
        join_all(self.tasks).await;
    }
}
