use candid::{CandidType, Decode, Encode, Principal};
use ic_cdk::api::management_canister::{
    bitcoin::BitcoinNetwork,
    ecdsa::{EcdsaCurve, EcdsaKeyId},
};
use ic_stable_structures::{StableCell, Storable, storable::Bound};
use serde::Deserialize;

use super::{Memory, MemoryIds, read_memory_manager};
use crate::EcdsaPublicKey;

#[derive(CandidType, Deserialize, Clone)]
pub struct Config {
    pub bitcoin_network: BitcoinNetwork,
    pub keyname: String,
    pub ecdsa_public_key: Option<EcdsaPublicKey>,
    pub initialized: bool,
    pub commission: u16,
    pub commission_receiver: String,
    pub factory: Principal,
    pub creator: String,
    pub runeid: String, // should be in format of `block:tx`. e.g. `840000:1`
    pub project_description: String,
    pub logo: String,
    pub system_prompt: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bitcoin_network: BitcoinNetwork::Regtest,
            keyname: String::from("dfx-test-key"),
            ecdsa_public_key: None,
            initialized: false,
            commission: 3,
            commission_receiver: String::new(),
            factory: Principal::anonymous(),
            creator: String::new(),
            runeid: String::new(),
            project_description: String::new(),
            logo: String::new(),
            system_prompt: String::new(),
        }
    }
}

impl Storable for Config {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).expect("should encode"))
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("should decode")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Config {
    pub fn ecdsa_public_key(&self) -> EcdsaPublicKey {
        if let Some(ref ecdsa_key) = self.ecdsa_public_key {
            ecdsa_key.clone()
        } else {
            ic_cdk::trap("canister's config uninitialized")
        }
    }

    pub fn keyname(&self) -> String {
        self.keyname.clone()
    }

    pub fn ecdsakeyid(&self) -> EcdsaKeyId {
        let name = self.keyname();
        EcdsaKeyId {
            name,
            curve: EcdsaCurve::Secp256k1,
        }
    }

    pub fn get_system_prompt(&self) -> String {
        self.system_prompt.clone()
    }

    pub fn get_logo(&self) -> String {
        self.logo.clone()
    }
}

pub type StableConfig = StableCell<Config, Memory>;

pub fn init_stable_config() -> StableConfig {
    read_memory_manager(|manager| {
        let memory = manager.get(MemoryIds::Config.into());
        StableConfig::new(memory, Config::default()).expect("should reserve memory for config")
    })
}
