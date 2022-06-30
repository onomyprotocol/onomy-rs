use std::{collections::HashMap, sync::Mutex};

use crate::EquityStorage;

#[derive(Debug, Default)]
pub struct InMemoryDb {
    data_holder: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
}

impl EquityStorage for InMemoryDb {
    fn get(&self, key: &[u8]) -> crate::DatabaseResult<Option<Vec<u8>>> {
        Ok(self
            .data_holder
            .lock()
            .expect("Lock is poisoned")
            .get(key)
            .cloned())
    }

    fn set(&self, key: Vec<u8>, value: Vec<u8>) -> crate::DatabaseResult<Option<Vec<u8>>> {
        Ok(self
            .data_holder
            .lock()
            .expect("Lock is Poisoned")
            .insert(key, value))
    }
}
