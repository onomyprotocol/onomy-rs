use std::{collections::BTreeMap, sync::Arc};

use futures::{SinkExt, StreamExt};

use serde_json;

use tokio_tungstenite::{connect_async, tungstenite::Message};

use tokio::sync::mpsc::{Sender, channel};

use equity_types::{ClientMsg, TransactionCommand};

use rand::Rng;

use tokio_stream::wrappers::ReceiverStream;

use tracing::info;

use credentials::{Credentials, Keys};

use crate::Error;

#[derive(Debug, Clone)]
pub struct EquityClient {
    pub sender: Sender<ClientMsg>,
    pub credentials: Arc<Credentials>
}

impl EquityClient {
    pub async fn new(ws_addr: &str, keys: Keys) -> Result<Self, Error> {
        
        let credentials = Credentials::new(keys);

        let (ws_stream, _) = connect_async(ws_addr)
        .await
        .expect("Failed to connect");

        println!("WebSocket connection established: {}", ws_addr);

        let (mut write, mut read) = ws_stream.split();

        // Insert the write part of this peer to the peer map.
        let (tx, rx) = channel::<ClientMsg>(1000);
        
        let mut rx = ReceiverStream::new(rx);

        tokio::spawn(async move {
            while let Some(msg) = rx.next().await {
                if let Err(e) = write.send(
                    Message::binary(
                        serde_json::to_vec(&msg).expect("msg does not have serde serialize trait"))
                    ).await {
                        println!("{:?}", e);
                    }
            }
        });
        
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                let data = msg.unwrap().into_data();
                let v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                println!("{:?}", v);
            }
        });

        let res = Self {
            sender: tx,
            credentials: Arc::new(credentials)
        };
        

        info!(target = "equity-client", "WS Address is: {:?}", ws_addr);
        Ok(res)
    }

    

    pub async fn health(&self) {
        
    }


    pub fn test_transaction(&self, key_domain: &u64, value_range: &u64, iterations: &u8) -> TransactionCommand {
        let mut rng = rand::thread_rng();

        // BTreeMap needed as keys are sorted vs HashMap
        let mut keys_values = BTreeMap::new();

        for _i in 0..*iterations {
            let o: u64 = rng.gen_range(0..*key_domain);
            let p: u64 = rng.gen_range(0..*value_range);
            keys_values.insert(o, p);
        }

        TransactionCommand::SetValues {
            keys_values
        }
    }

    pub async fn send_transaction(
        &self,
        transaction: ClientMsg,
    ) {
      self.sender.send(transaction).await.expect("Channel failed");  
    }
}
