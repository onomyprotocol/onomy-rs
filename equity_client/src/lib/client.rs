use std::{
    io,
    str::FromStr,
    time::{Duration, Instant},
};

use borsh::BorshDeserialize;
use equity_types::{EquityAddressResponse, HealthResponse};
use surf::Url;
use tokio::time::sleep;
use tracing::info;

use crate::Error;

pub struct EquityClient {
    surf_url: Url,
    url_health: Url,
    url_address: Url,
}

impl EquityClient {
    pub fn new(url: &str) -> Result<Self, Error> {
        let s_url = Url::from_str(url)?;
        let res = Self {
            url_health: s_url.join("health").unwrap(),
            url_address: s_url.join("address").unwrap(),
            surf_url: s_url,
        };
        info!(target = "equity-client", "URL is: {:?}", res.surf_url);
        Ok(res)
    }

    /// Waits until the client successfully gets a healthy response
    pub async fn wait_for_healthy(&self, timeout: Duration) -> crate::Result<()> {
        let end = Instant::now() + timeout;
        while Instant::now() < end {
            if let Ok(response) = surf::get(&self.url_health).recv_bytes().await {
                let health: HealthResponse =
                    BorshDeserialize::try_from_slice(&response).map_err(Error::from)?;
                if health.up {
                    return Ok(())
                }
            }
            sleep(Duration::from_millis(500)).await
        }
        Err(Error::from(io::Error::from(io::ErrorKind::TimedOut)))
    }

    pub async fn health(&self) -> crate::Result<HealthResponse> {
        BorshDeserialize::try_from_slice(&surf::get(&self.url_health).recv_bytes().await.unwrap())
            .map_err(Error::from)
    }

    pub async fn get_account_details(
        &self,
        address: String,
    ) -> crate::Result<EquityAddressResponse> {
        BorshDeserialize::try_from_slice(
            &surf::get(self.url_address.join(&address)?)
                .recv_bytes()
                .await?,
        )
        .map_err(Error::from)
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
