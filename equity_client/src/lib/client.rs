use std::{
    io,
    str::FromStr,
    time::{Duration, Instant},
};

use borsh::BorshDeserialize;
use equity_types::{EquityAddressResponse, HealthResponse};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use surf::Url;
use tokio::time::sleep;
use tracing::info;
use std::collections::HashMap;
use rand::{Rng, thread_rng};

use ed25519_consensus::{Signature, SigningKey, VerificationKey};
use sha2::{Digest, Sha512};

use crate::Error;


pub struct EquityClient {
    surf_url: Url,
    url_health: String,
    url_address: String,
    private_key: SigningKey,
    public_key: VerificationKey
}

/// Includes along with the real `body` message a hash and signature
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct FullMessage {
    body: String,
    hash: String,
    signature: Signature,
}

pub async fn borsh_get<T: BorshDeserialize>(url: &Url) -> crate::Result<T> {
    let response = surf::get(url).recv_bytes().await?;
    BorshDeserialize::try_from_slice(&response)
        .map_err(|e| Error::BorshDeserializeError(e, response))
}

/// Used for message debugging
pub async fn ron_get<T: DeserializeOwned>(url: &Url) -> crate::Result<T> {
    let response = surf::get(url).recv_bytes().await?;
    ron::de::from_bytes(&response).map_err(|e| Error::RonDeserializeError(e, response))
}

impl EquityClient {
    pub fn new(url: &str) -> Result<Self, Error> {
        let s_url = Url::from_str(url)?;
        let sk = SigningKey::new(thread_rng());
        let vk = VerificationKey::from(&sk);

        let res = Self {
            surf_url: s_url,
            url_health: "health".to_owned(),
            url_address: "address/".to_owned(),
            private_key: sk,
            public_key: vk
        };
        info!(target = "equity-client", "URL is: {:?}", res.surf_url);
        Ok(res)
    }

    /// Waits until the client successfully gets a healthy response
    pub async fn wait_for_healthy(&self, timeout: Duration) -> crate::Result<()> {
        let end = Instant::now() + timeout;
        while Instant::now() < end {
            if let Ok(response) = surf::get(&self.surf_url.join(&self.url_health)?)
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

    pub fn test_transaction(key_domain: &u64, value_range: &u64, iterations: &u8) -> String {
        let mut rng = rand::thread_rng();

        let mut keys_values = HashMap::new();
        for _i in 0..*iterations {
            let o: u64 = rng.gen_range(0..*key_domain);
            let p: u64 = rng.gen_range(0..*value_range);
            keys_values.insert(o, p);
        }

        serde_json::to_string(&keys_values).unwrap()
    }

    pub fn create_transaction(&self, message: &String) -> () {
        
        println!("message: {}", message);

        // Create a hash digest object which we'll feed the message into:
        let mut digest: Sha512 = Sha512::new();
        digest.update(message);

        let digest_string: String = format!("{:X}", digest.clone().finalize());
    
        let signature: Signature = self.private_key.sign(&digest_string.as_bytes());
    
        let network_message0 = FullMessage {
            body: message.to_string(),
            hash: digest_string.clone(),
            signature,
        };
    
        println!(
            "network message: {}",
            serde_json::to_string_pretty(&network_message0).unwrap()
        );
    
        let s = serde_json::to_string(&network_message0).unwrap();
    
        // (s can be sent over a channel)
    
        let network_message1 = serde_json::from_str(&s).unwrap();
        assert_eq!(network_message0, network_message1);
    
        // TODO need wrapper for SHA512 or maybe we shouldn't be sending the hash over
        // network, and instead using the plain `sign/verify`
        self
            .public_key
            .verify(&signature.into(), &digest_string.as_bytes());
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
