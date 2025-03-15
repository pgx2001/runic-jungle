use crate::EcdsaPublicKey;
use candid::{CandidType, Decode, Encode, Principal};
use ic_cdk::api::management_canister::{
    bitcoin::BitcoinNetwork,
    ecdsa::{EcdsaCurve, EcdsaKeyId},
};
use ic_stable_structures::{StableCell, Storable, storable::Bound};
use serde::Deserialize;

use super::{CanisterMemory, CanisterMemoryIds, read_memory_manager};

#[derive(CandidType, Deserialize, Clone)]
pub struct Config {
    pub bitcoin_network: BitcoinNetwork,
    pub auth: Option<Principal>,
    pub commission_receiver: Option<String>,
    pub creation_fee: u64, // defaults to 20_000 satoshis
    pub commission: u16,   // defaults to 2%
    pub ecdsa_public_key: Option<EcdsaPublicKey>,
    pub keyname: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bitcoin_network: BitcoinNetwork::Regtest,
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
        std::borrow::Cow::Owned(Encode!(self).expect("should encode"))
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("should decode")
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

impl Config {
    pub fn bitcoin_network(&self) -> BitcoinNetwork {
        self.bitcoin_network
    }

    pub fn ecdsa_public_key(&self) -> EcdsaPublicKey {
        if let Some(ref ecdsa_key) = self.ecdsa_public_key {
            ecdsa_key.clone()
        } else {
            ic_cdk::trap("canister's config uninitialized")
        }
    }

    pub fn commission_receiver(&self) -> String {
        self.commission_receiver.clone().unwrap_or_else(|| {
            crate::bitcoin::account_to_p2pkh_address(&icrc_ledger_types::icrc1::account::Account {
                owner: ic_cdk::id(),
                subaccount: None,
            })
        })
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
}
