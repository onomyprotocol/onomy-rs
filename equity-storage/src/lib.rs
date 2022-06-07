use borsh::{BorshDeserialize, BorshSerialize};
use std::{fmt::Debug, sync::Arc};
use thiserror::Error;

pub type DatabaseResult<T> = Result<T, Error>;
mod in_memory;

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

    pub fn get<V: BorshDeserialize>(&self, key: &[u8]) -> Result<Option<V>, Error> {
        self.data
            .get(key)?
            .map(|val| BorshDeserialize::try_from_slice(&val).map_err(|_| Error::Codec))
            .transpose()
    }

    pub fn insert<K: Into<Vec<u8>>, V: BorshSerialize + BorshDeserialize>(
        &self,
        key: K,
        value: V,
    ) -> Result<Option<V>, Error> {
        match self
            .data
            .insert(key.into(), borsh::to_vec(&value).map_err(|_| Error::Codec)?)?
        {
            Some(previous_value) => match BorshDeserialize::try_from_slice(&previous_value) {
                Ok(o) => Ok(Some(o)),
                Err(_) => Err(Error::Codec),
            },
            _ => Ok(None),
        }
    }
}

pub enum DatabaseType {
    InMemory,
}

pub trait EquityStorage: Debug + Send + Sync {
    fn get(&self, key: &[u8]) -> DatabaseResult<Option<Vec<u8>>>;
    /// Inserts `key` and corresponding `value` into the database. If an entry with `key` already existed, the previous value is returned
    fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> DatabaseResult<Option<Vec<u8>>>;
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error during binary serialization")]
    Codec,
    #[error("Database Error `{0}`")]
    DatabaseError(Box<dyn std::error::Error + Send + Sync>),
}
