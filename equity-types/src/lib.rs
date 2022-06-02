use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct EquityTx {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct EquityAddressResponse {
    pub owner: String,
    pub value: u64,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub up: bool,
}
