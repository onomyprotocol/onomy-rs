pub use borsh;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Default, Debug, BorshSerialize, BorshDeserialize)]
pub struct EquityTx {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Default, Debug, BorshSerialize, BorshDeserialize)]
pub struct EquityAddressResponse {
    pub owner: String,
    pub value: u64,
}

#[derive(Default, Debug, BorshSerialize, BorshDeserialize)]
pub struct HealthResponse {
    pub up: bool,
}
