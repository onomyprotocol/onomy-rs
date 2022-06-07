use std::str::FromStr;

use borsh::BorshDeserialize;
use equity_types::{EquityAddressResponse, HealthResponse};
use tracing::info;

pub struct EquityClient {
    url: String,
}

impl EquityClient {
    pub fn new(url: impl AsRef<str>) -> Result<Self, anyhow::Error> {
        Self::from_str(url.as_ref())
    }

    pub async fn health(&self) -> std::io::Result<HealthResponse> {
        let url = surf::Url::from_str(&format!("{}/health", self.url)).unwrap();
        info!(target = "equity-client", "URL is: {:?}", url);
        let request: Result<HealthResponse, _> =
            BorshDeserialize::try_from_slice(&surf::get(url).recv_bytes().await.unwrap());

        match request {
            Ok(response) => Ok(response),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    pub async fn get_account_details(
        &self,
        address: String,
    ) -> std::io::Result<EquityAddressResponse> {
        let url = surf::Url::from_str(&format!("{}/address/{}", self.url, address)).unwrap();
        info!(target = "equity-client", "URL is: {:?}", url);
        let query = BorshDeserialize::try_from_slice(&surf::get(url).recv_bytes().await.unwrap());

        match query {
            Ok(response) => Ok(response),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}

impl FromStr for EquityClient {
    type Err = anyhow::Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            url: str.to_owned(),
        })
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
