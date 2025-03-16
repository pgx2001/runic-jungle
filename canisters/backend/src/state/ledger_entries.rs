use ic_stable_structures::{StableBTreeMap, Storable, storable::Bound};
use std::collections::HashMap;

use super::{CanisterMemory, CanisterMemoryIds, read_memory_manager};
use crate::indexer::RuneId;
use candid::{CandidType, Decode, Deserialize, Encode, Principal};

/* #[derive(CandidType, Deserialize)]
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
} */

#[derive(CandidType, Deserialize)]
pub struct BalanceEntries {
    pub restricted_bitcoin_balance: u64,
    pub ledger_entries: HashMap<u128, (u64, u128)>, // mapping of agent_id to (bitcoin owned by ageint, rune balance of user)
}

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
    pub fn record_rune_balance(&mut self, agent: u128, rune: u128) {
        let entry = self.ledger_entries.entry(agent).or_default();
        entry.1 += rune;
    }

    pub fn record_agent_owned_balance(&mut self, agent: u128, bitcoin: u64) {
        self.restricted_bitcoin_balance += bitcoin;
        let entry = self.ledger_entries.entry(agent).or_default();
        entry.0 += bitcoin;
    }

    pub fn deduct_rune_balance(&mut self, agent: u128, rune: u128) {
        let entry = self.ledger_entries.get_mut(&agent).expect("should exist");
        entry.1 -= rune;
    }

    pub fn deduct_agent_owned_balance(&mut self, agent: u128, bitcoin: u64) {
        self.restricted_bitcoin_balance -= bitcoin;
        let entry = self.ledger_entries.get_mut(&agent).expect("should exist");
        entry.0 -= bitcoin;
    }
}

pub type LedgerEntries = StableBTreeMap<Principal, BalanceEntries, CanisterMemory>;

pub fn init_ledger_entries() -> LedgerEntries {
    read_memory_manager(|manager| {
        let memory = manager.get(CanisterMemoryIds::LedgerEntries.into());
        LedgerEntries::init(memory)
    })
}
