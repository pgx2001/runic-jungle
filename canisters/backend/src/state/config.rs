use crate::EcdsaPublicKey;
use candid::{CandidType, Principal};
use ic_stable_structures::{StableCell, Storable, storable::Bound};
use serde::Deserialize;

use super::{CanisterMemory, CanisterMemoryIds, read_memory_manager};

#[derive(CandidType, Deserialize)]
pub struct Config {
    pub auth: Option<Principal>,
    pub commission_receiver: Option<String>,
    pub creation_fee: u16, // defaults to 20_000 satoshis
    pub commission: u16,   // defaults to 2%
    pub ecdsa_public_key: Option<EcdsaPublicKey>,
    pub keyname: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auth: None,
            commission_receiver: None,
            creation_fee: 20_000,
            commission: 200,
            ecdsa_public_key: None,
            keyname: String::from("dfx_test_key"),
        }
    }
}

impl Storable for Config {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        todo!()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        todo!()
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub type StableConfig = StableCell<Config, CanisterMemory>;

pub fn initialize_config() -> StableConfig {
    read_memory_manager(|manager| {
        let memory = manager.get(CanisterMemoryIds::Config.into());
        StableConfig::init(memory, Config::default()).expect("should initialize config")
    })
}

impl Config {}
