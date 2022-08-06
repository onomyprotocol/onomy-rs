use std::collections::BTreeMap;

use futures::{SinkExt, StreamExt};

use serde_json;

use tokio_tungstenite::{connect_async, tungstenite::Message};

use tokio::sync::mpsc::{Sender, channel};

use equity_types::{Credentials, ClientCommand, TransactionBody};

use rand::Rng;

use tokio_stream::wrappers::ReceiverStream;

use tracing::info;

use crate::Error;

pub struct EquityClient {
    sender: Sender<ClientCommand>,
    credentials: Credentials,
    nonce: u64
}

impl EquityClient {
    pub async fn new(url: &str) -> Result<Self, Error> {
        
        let (ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect");

        println!("WebSocket connection established: {}", url);

        let (mut write, mut read) = ws_stream.split();

        let credentials = Credentials::new();

        // Insert the write part of this peer to the peer map.
        let (tx, rx) = channel::<ClientCommand>(1000);
        
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
            credentials,
            nonce: 1 
        };
        

        info!(target = "equity-client", "URL is: {:?}", url);
        Ok(res)
    }

    pub fn noncer(&mut self) {
        self.nonce += 1;
    }

    pub async fn health(&self) {
        
    }


    pub fn test_transaction(&self, key_domain: &u64, value_range: &u64, iterations: &u8) -> TransactionBody {
        let mut rng = rand::thread_rng();

        // BTreeMap needed as keys are sorted vs HashMap
        let mut keys_values = BTreeMap::new();

        for _i in 0..*iterations {
            let o: u64 = rng.gen_range(0..*key_domain);
            let p: u64 = rng.gen_range(0..*value_range);
            keys_values.insert(o, p);
        }

        TransactionBody::SetValues {
            public_key: self.credentials.public_key,
            nonce: self.nonce,
            keys_values
        }
    }

    pub fn create_transaction(&self, message: &TransactionBody) -> ClientCommand {
        let message_string = serde_json::to_string(message).unwrap();

        let (digest_string, signature) = self.credentials.hash_sign(&message_string);

        ClientCommand::Transaction {
            body: message.clone(),
            hash: digest_string,
            signature,
        }
    }

    pub async fn send_transaction(
        &self,
        transaction: ClientCommand,
    ) {
      self.sender.send(transaction).await.expect("Channel failed");  
    }
}
