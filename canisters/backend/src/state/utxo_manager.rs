use std::collections::{HashMap, HashSet};

use candid::{CandidType, Decode, Encode};
use ic_cdk::api::management_canister::bitcoin::Utxo;
use ic_stable_structures::{StableBTreeMap, Storable, storable::Bound};
use serde::{Deserialize, Serialize};

use crate::indexer::RuneId;

use super::{CanisterMemory, CanisterMemoryIds, read_memory_manager};

#[derive(CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct RunicUtxo {
    pub balance: u128,
    pub utxo: Utxo,
}

impl std::hash::Hash for RunicUtxo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.utxo.hash(state)
    }
}

impl std::borrow::Borrow<Utxo> for RunicUtxo {
    fn borrow(&self) -> &Utxo {
        &self.utxo
    }
}

#[derive(CandidType, Deserialize, Default)]
pub struct RunicToUtxoMapping(HashMap<RuneId, HashSet<RunicUtxo>>);

impl Storable for RunicToUtxoMapping {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).expect("should encode"))
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("should decode")
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub type RunicMapping = StableBTreeMap<String, RunicToUtxoMapping, CanisterMemory>;

pub fn init_runic_mapping() -> RunicMapping {
    read_memory_manager(|manager| {
        let memory = manager.get(CanisterMemoryIds::Runic.into());
        RunicMapping::init(memory)
    })
}

#[derive(CandidType, Deserialize, Default)]
pub struct Utxos(HashSet<Utxo>);

impl Storable for Utxos {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).expect("should encode"))
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("should decode")
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub type BitcoinMapping = StableBTreeMap<String, Utxos, CanisterMemory>;

pub fn init_bitcoin_mapping() -> BitcoinMapping {
    read_memory_manager(|manager| {
        let memory = manager.get(CanisterMemoryIds::Bitcoin.into());
        BitcoinMapping::init(memory)
    })
}

#[derive(Serialize, Deserialize)]
pub struct UtxoManager {
    #[serde(skip, default = "init_runic_mapping")]
    pub runic: RunicMapping,
    #[serde(skip, default = "init_bitcoin_mapping")]
    pub bitcoin: BitcoinMapping,
}

impl Default for UtxoManager {
    fn default() -> Self {
        Self {
            runic: init_runic_mapping(),
            bitcoin: init_bitcoin_mapping(),
        }
    }
}

impl UtxoManager {
    pub fn record_runic_utxos(&mut self, addr: &str, runeid: RuneId, utxos: Vec<RunicUtxo>) {
        let addr = String::from(addr);
        let mut map = self.runic.get(&addr).unwrap_or_default().0;
        let mut current_utxos = map.remove(&runeid).unwrap_or_default();
        for utxo in utxos {
            if current_utxos.contains(&utxo) {
                continue;
            }
            current_utxos.insert(utxo);
        }
        map.insert(runeid, current_utxos);
        self.runic.insert(addr, RunicToUtxoMapping(map));
    }

    pub fn record_bitcoin_utxos(&mut self, addr: &str, utxos: Vec<Utxo>) {
        let addr = String::from(addr);
        let mut current_utxos = self.bitcoin.get(&addr).unwrap_or_default().0;
        for utxo in utxos {
            if current_utxos.contains(&utxo) {
                continue;
            }
            current_utxos.insert(utxo);
        }
        self.bitcoin.insert(addr, Utxos(current_utxos));
    }

    pub fn get_bitcoin_utxo(&mut self, addr: &str) -> Option<Utxo> {
        let addr = String::from(addr);
        let mut utxos = self.bitcoin.get(&addr)?.0;
        let min_utxo = utxos.iter().min_by_key(|utxo| utxo.value)?.clone();
        utxos.remove(&min_utxo);
        self.bitcoin.insert(addr, Utxos(utxos));
        Some(min_utxo)
    }

    pub fn get_runic_utxo(&mut self, addr: &str, runeid: RuneId) -> Option<RunicUtxo> {
        let addr = String::from(addr);
        let mut map = self.runic.get(&addr)?.0;
        let mut utxos = map.remove(&runeid).unwrap_or_default();
        let min_utxo = utxos.iter().min_by_key(|utxo| utxo.balance)?.clone();
        utxos.remove(&min_utxo);
        map.insert(runeid, utxos);
        self.runic.insert(addr, RunicToUtxoMapping(map));
        Some(min_utxo)
    }

    pub fn is_recorded_as_runic(&self, addr: &str, utxo: &Utxo) -> bool {
        let addr = String::from(addr);
        let mut flag = false;
        if let Some(map) = self.runic.get(&addr) {
            for (_, utxos) in map.0.iter() {
                if utxos.contains(utxo) {
                    flag = true;
                    break;
                }
            }
        }
        flag
    }

    pub fn get_runestone_balance(&self, addr: &str, runeid: &RuneId) -> u128 {
        let addr = String::from(addr);
        let mut balance = 0;
        if let Some(map) = self.runic.get(&addr) {
            if let Some(utxos) = map.0.get(runeid) {
                balance = utxos.iter().fold(0, |balance, utxo| balance + utxo.balance);
            }
        }
        balance
    }

    pub fn get_bitcoin_balance(&self, addr: &str) -> u64 {
        let addr = String::from(addr);
        let mut balance = 0;
        if let Some(utxos) = self.bitcoin.get(&addr) {
            balance = utxos.0.iter().fold(0, |balance, utxo| balance + utxo.value);
        }
        balance
    }

    pub fn all_rune_with_balances(&self, addr: &str) -> HashMap<RuneId, u128> {
        let addr = String::from(addr);
        let mut balances = HashMap::new();
        if let Some(map) = self.runic.get(&addr) {
            for (r, utxos) in map.0.iter() {
                let balance = utxos.iter().fold(0, |balance, utxo| balance + utxo.balance);
                balances.insert(*r, balance);
            }
        }
        balances
    }

    pub fn remove_bitcoin_utxo(&mut self, addr: &str, utxo: &Utxo) {
        let addr = String::from(addr);
        let mut current_utxos = self.bitcoin.get(&addr).unwrap_or_default().0;
        current_utxos.remove(utxo);
        self.bitcoin.insert(addr, Utxos(current_utxos));
    }
}
