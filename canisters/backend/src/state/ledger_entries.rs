use ic_stable_structures::{StableBTreeMap, Storable, storable::Bound};
use std::collections::HashSet;

use super::{CanisterMemory, CanisterMemoryIds, read_memory_manager};
use candid::{CandidType, Decode, Deserialize, Encode, Principal};

#[derive(CandidType, Deserialize)]
pub struct BalanceEntries(HashSet<u128>);

impl Storable for BalanceEntries {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).expect("should encode"))
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("should decode")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl BalanceEntries {
    pub fn record_entry(&mut self, entry: u128) {
        if !self.0.contains(&entry) {
            self.0.insert(entry);
        }
    }
}

pub type LedgerEntries = StableBTreeMap<Principal, BalanceEntries, CanisterMemory>;

pub fn init_ledger_entries() -> LedgerEntries {
    read_memory_manager(|manager| {
        let memory = manager.get(CanisterMemoryIds::LedgerEntries.into());
        LedgerEntries::init(memory)
    })
}
