use crate::bitcoin_lib::Transaction;
use ic_stable_structures::{StableBTreeMap, Storable, storable::Bound};
use serde::{Deserialize, Serialize};
use slotmap::KeyData;

use super::{CanisterMemory, CanisterMemoryIds, read_memory_manager};

#[derive(Serialize, Deserialize)]
pub struct ScheduledTransaction {
    pub commit_tx_address: String,
    pub txn: Transaction,
    pub timer_id: KeyData,
}

impl Storable for ScheduledTransaction {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(&self, &mut bytes).unwrap();
        std::borrow::Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        ciborium::de::from_reader(&*bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub type ScheduledTransactionMap = StableBTreeMap<u128, ScheduledTransaction, CanisterMemory>;

fn init_mapping() -> ScheduledTransactionMap {
    read_memory_manager(|manager| {
        let memory = manager.get(CanisterMemoryIds::Queue.into());
        ScheduledTransactionMap::init(memory)
    })
}

#[derive(Serialize, Deserialize)]
pub struct ScheduledState {
    txn_count: u128,
    #[serde(skip, default = "init_mapping")]
    mapping: ScheduledTransactionMap,
}

impl Default for ScheduledState {
    fn default() -> Self {
        Self {
            txn_count: 0,
            mapping: init_mapping(),
        }
    }
}

impl ScheduledState {
    pub fn get_id(&mut self) -> u128 {
        let id = self.txn_count;
        self.txn_count += 1;
        id
    }
    pub fn record_txn(&mut self, id: u128, txn: ScheduledTransaction) {
        self.mapping.insert(id, txn);
    }

    pub fn remove_txn(&mut self, key: u128) -> ScheduledTransaction {
        self.mapping
            .remove(&key)
            .expect("should remove the Transaction")
    }
}
