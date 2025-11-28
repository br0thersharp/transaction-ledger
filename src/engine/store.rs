use crate::core::types::*;
use crate::engine::state::TxRecord;
use std::collections::HashMap;

pub trait TxStore {
    fn get(&self, tx: TxId) -> Option<&TxRecord>;
    fn get_mut(&mut self, tx: TxId) -> Option<&mut TxRecord>;
    fn insert(&mut self, tx: TxId, rec: TxRecord);
    fn contains(&self, tx: TxId) -> bool;
}

#[derive(Debug, Default)]
pub struct HashMapStore {
    inner: HashMap<TxId, TxRecord>,
}

impl HashMapStore {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

impl TxStore for HashMapStore {
    fn get(&self, tx: TxId) -> Option<&TxRecord> {
        self.inner.get(&tx)
    }
    fn get_mut(&mut self, tx: TxId) -> Option<&mut TxRecord> {
        self.inner.get_mut(&tx)
    }
    fn insert(&mut self, tx: TxId, rec: TxRecord) {
        self.inner.insert(tx, rec);
    }
    fn contains(&self, tx: TxId) -> bool {
        self.inner.contains_key(&tx)
    }
}
