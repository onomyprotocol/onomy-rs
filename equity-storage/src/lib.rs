use serde::{de::DeserializeOwned, Serialize};
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

    pub fn get<V: DeserializeOwned>(&self, key: &[u8]) -> Result<Option<V>, Error> {
        self.data
            .get(key)?
            .map(|val| bincode::deserialize(&val).map_err(|_| Error::Codec))
            .transpose()
    }

    pub fn insert<K: Into<Vec<u8>>, V: Serialize + DeserializeOwned>(
        &self,
        key: K,
        value: V,
    ) -> Result<Option<V>, Error> {
        match self.data.insert(
            key.into(),
            bincode::serialize(&value).map_err(|_| Error::Codec)?,
        )? {
            Some(previous_value) => Ok(Some(
                bincode::deserialize(&previous_value).map_err(|_| Error::Codec)?,
            )),
            _ => Ok(None),
        }
    }
}

pub enum DatabaseType {
    InMemory,
}

pub trait EquityStorage: Debug + Send + Sync {
    fn get(&self, key: &[u8]) -> DatabaseResult<Option<Vec<u8>>>;
    fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> DatabaseResult<Option<Vec<u8>>>;
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error during binary serialization")]
    Codec,
    #[error("Database Error `{0}`")]
    DatabaseError(Box<dyn std::error::Error + Send + Sync>),
}
