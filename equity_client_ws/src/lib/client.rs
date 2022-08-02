use std::{
    collections::BTreeMap,
    io,
    str::FromStr,
    time::{Duration, Instant},
};

use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::channel,
    task::JoinHandle,
};

use equity_types::{
    Body, Credentials, EquityAddressResponse, FullMessage, HealthResponse, PostTransactionResponse,
};


use rand::Rng;
use serde::de::DeserializeOwned;

use tokio::{
    time::sleep,
    io::stdout
};

use tokio_stream::wrappers::ReceiverStream;

use tracing::info;

use crate::Error;

pub struct EquityClient {
    send: Sender,
    credentials: Credentials,
    nonce: u64,
}

impl EquityClient {
    pub fn new(url: &str) -> Result<Self, Error> {
        
        let (mut ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect");

        println!("WebSocket handshake has been successfully completed");

        let (write, read) = ws_stream.split();

        let credentials = Credentials::new();

        // Insert the write part of this peer to the peer map.
        let (tx, rx) = channel(1000);
        let rx = ReceiverStream::new(rx);

        tokio::spawn(rx.map(Ok).forward(write));

        let res = Self {
            send: tx,
            credentials,
            nonce: 1,
        };
        
        tokio::spawn(
            while let Some(msg) = read.next().await {
                let data = msg.unwrap().into_data();
                stdout().write_all(&data).await.unwrap();
            }
        );
        

        info!(target = "equity-client", "URL is: {:?}", url);
        Ok(res)
    }

    pub fn noncer(&mut self) {
        self.nonce += 1;
    }

    /// Waits until the client successfully gets a healthy response
    pub async fn wait_for_healthy(&self, timeout: Duration) -> crate::Result<()> {
        let end = Instant::now() + timeout;
        while Instant::now() < end {
            if let Ok(response) = ws.send(&self.surf_url.join(&self.url_health)?)
                .recv_bytes()
                .await
            {
                let health: HealthResponse = BorshDeserialize::try_from_slice(&response)
                    .map_err(|e| Error::BorshDeserializeError(e, response))?;
                if health.up {
                    return Ok(())
                }
            }
            sleep(Duration::from_millis(500)).await
        }
        Err(Error::StdIoError(io::Error::from(io::ErrorKind::TimedOut)))
    }

    pub async fn health(&self) -> crate::Result<HealthResponse> {
        borsh_get(&self.surf_url.join(&self.url_health)?).await
    }

    pub async fn get_account_details(&self, address: &str) -> crate::Result<EquityAddressResponse> {
        borsh_get(
            &self
                .surf_url
                .join(&format!("{}{}", self.url_address, address))?,
        )
        .await
    }

    pub fn test_transaction(&self, key_domain: &u64, value_range: &u64, iterations: &u8) -> Body {
        let mut rng = rand::thread_rng();

        // BTreeMap needed as keys are sorted vs HashMap
        let mut keys_values = BTreeMap::new();
        for _i in 0..*iterations {
            let o: u64 = rng.gen_range(0..*key_domain);
            let p: u64 = rng.gen_range(0..*value_range);
            keys_values.insert(o, p);
        }

        Body {
            public_key: self.credentials.public_key,
            nonce: self.nonce,
            keys_values,
        }
    }

    pub fn create_transaction(&self, message: &Body) -> FullMessage {
        let message_string = serde_json::to_string(message).unwrap();

        let (digest_string, signature) = self.credentials.hash_sign(&message_string);

        FullMessage {
            body: message.clone(),
            hash: digest_string,
            signature,
        }
    }

    pub async fn post_transaction(
        &self,
        transaction: FullMessage,
    ) -> crate::Result<PostTransactionResponse> {
        let mut url = self.surf_url.clone();
        url.set_path(&self.url_transaction);
        serde_post(&url.join(&transaction.hash)?, transaction).await
    }
}

impl FromStr for EquityClient {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl<S> From<S> for EquityClient
where
    S: Into<std::net::SocketAddr>,
{
    fn from(socket: S) -> Self {
        format!("http://{}", socket.into())
            .as_str()
            .parse()
            .unwrap()
    }
}
