mod in_memory;
use std::{fmt::Debug, sync::Arc};

use serde::{Serialize};
use serde::de::DeserializeOwned;
use serde_json::*;

pub enum DatabaseType {
    InMemory,
}

pub trait EquityStorage: Debug + Send + Sync {
    fn get(&self, key: &[u8]) -> DatabaseResult<Option<Vec<u8>>>;
    /// Sets `key` and corresponding `value` into the database. If an entry
    /// with `key` already existed, the previous value is returned
    fn set(&self, key: Vec<u8>, value: Vec<u8>) -> DatabaseResult<Option<Vec<u8>>>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(
        "Error during binary serialization, you probably have the wrong type on the receiving end \
         or inserted an invalid repr into a database"
    )]
    Codec,
    #[error("Database Error `{0}`")]
    DatabaseError(Box<dyn std::error::Error + Send + Sync>),
}

pub type DatabaseResult<T> = Result<T>;

#[derive(Clone, Debug)]
pub struct EquityDatabase {
    data: Arc<dyn EquityStorage>,
}

impl EquityDatabase {
    pub fn in_memory() -> Self {
        Self {
            data: Arc::new(in_memory::InMemoryDb::default()),
        }
    }

    pub fn get<V: DeserializeOwned + Debug>(&self, key: &[u8]) -> Result<Option<V>> {
        match self.data.get(key) {
            Ok(Some(bytes)) => {
                let val: V = serde_json::from_slice(&bytes)?;
                Ok(Some(val))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set<K: Serialize, V: Serialize + DeserializeOwned>(
        &self,
        key: K,
        value: V,
    ) -> Result<Option<V>> {
        match self
            .data
            .set(serde_json::to_vec(&key)?, serde_json::to_vec(&value)?)
        {
            Ok(Some(previous_value)) => {
                let val: V = serde_json::from_slice(&previous_value)?;
                Ok(Some(val))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
