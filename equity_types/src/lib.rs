pub use borsh;
use borsh::{BorshDeserialize, BorshSerialize};
use derive_alias::derive_alias;
use serde::{Deserialize, Serialize};

// TODO common derive macro

derive_alias! {
    derive_common => #[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord,
        BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
}

derive_common! {
pub struct EquityTx {
    pub from: String,
    pub to: String,
    pub amount: u64,
}
}

derive_common! {
pub struct Value(pub u64);
}

derive_common! {
pub struct EquityAddressResponse {
    pub owner: String,
    pub value: Value,
}
}

derive_common! {
pub struct EquityTransactionResponse {
    
}
}

derive_common! {
pub struct HealthResponse {
    pub up: bool,
}
}
