use crate::bitcoin_lib::hashes::Hash;
use crate::state::{read_config, read_utxo_manager, utxo_manager::RunicUtxo, write_utxo_manager};
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_cdk::api::management_canister::bitcoin::{GetUtxosRequest, UtxoFilter, bitcoin_get_utxos};
use ic_stable_structures::{Storable, storable::Bound};
use std::str::FromStr;

#[derive(CandidType, Deserialize, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Copy)]
pub struct RuneId {
    pub block: u64,
    pub tx: u32,
}

impl Storable for RuneId {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).expect("should encode"))
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("should decode")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl ToString for RuneId {
    fn to_string(&self) -> String {
        format!("{}:{}", self.block, self.tx)
    }
}

impl FromStr for RuneId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (block, tx) = s
            .split_once(":")
            .ok_or_else(|| String::from("Invalid Data"))?;
        let block = block
            .trim()
            .parse()
            .ok()
            .ok_or_else(|| String::from("Failed to parse"))?;

        let tx = tx
            .trim()
            .parse()
            .ok()
            .ok_or_else(|| String::from("Failed to parse"))?;
        Ok(Self { block, tx })
    }
}

pub mod runes_indexer {
    use candid::Principal;
    use runes_indexer_interface::{Error, RuneBalance, RuneEntry};

    const RUNES_INDEXER: &str = "f2dwm-caaaa-aaaao-qjxlq-cai";

    pub async fn get_rune(name: String) -> Option<RuneEntry> {
        ic_cdk::call::<(String,), (Option<RuneEntry>,)>(
            Principal::from_text(RUNES_INDEXER).unwrap(),
            "get_rune",
            (name,),
        )
        .await
        .unwrap()
        .0
    }

    pub async fn get_rune_by_id(name: String) -> Option<RuneEntry> {
        ic_cdk::call::<(String,), (Option<RuneEntry>,)>(
            Principal::from_text(RUNES_INDEXER).unwrap(),
            "get_rune_by_id",
            (name,),
        )
        .await
        .unwrap()
        .0
    }

    pub async fn get_rune_balances_for_outputs(
        outputs: Vec<String>,
    ) -> Result<Vec<Option<Vec<RuneBalance>>>, Error> {
        ic_cdk::call::<(Vec<String>,), (Result<Vec<Option<Vec<RuneBalance>>>, Error>,)>(
            Principal::from_text(RUNES_INDEXER).unwrap(),
            "get_rune_balances_for_outputs",
            (outputs,),
        )
        .await
        .unwrap()
        .0
    }
}

fn txid_to_string(txid: &[u8]) -> String {
    bitcoin::Txid::from_raw_hash(Hash::from_slice(txid).unwrap()).to_string()
}

pub enum TargetType {
    Bitcoin { target: u64 },
    Runic { runeid: RuneId, target: u128 },
}

pub async fn fetch_utxos_and_update(addr: &str, target: TargetType) {
    let network = read_config(|config| config.bitcoin_network());
    let mut arg = GetUtxosRequest {
        address: addr.to_string(),
        network,
        filter: None,
    };

    loop {
        let utxo_response = bitcoin_get_utxos(arg.clone())
            .await
            .expect("fetching utxo failed")
            .0;

        let mut requesting_utxos: Vec<String> = Vec::with_capacity(utxo_response.utxos.len());

        for utxo in utxo_response.utxos.iter() {
            // check if already recorded
            if read_utxo_manager(|manager| manager.is_recorded_as_runic(addr, &utxo)) {
                continue;
            }
            let txid = txid_to_string(&utxo.outpoint.txid);
            let txid_with_vout = format!("{txid}:{}", utxo.outpoint.vout);
            requesting_utxos.push(txid_with_vout);
        }

        /* match runes_indexer::get_rune_balances_for_outputs(requesting_utxos).await {
            Err(_) => {
                ic_cdk::println!(
                    "failed to fetch the rune balances. Recording everything as Bitcoin UTXOS"
                );
                write_utxo_manager(|manager| {
                    manager.record_bitcoin_utxos(addr, utxo_response.utxos)
                });
                break;
            }
            Ok(balances) => {
                let mut bitcoin_utxos = vec![];
                for (utxo, rune_balance) in utxo_response.utxos.into_iter().zip(balances) {
                    match rune_balance {
                        None => {
                            bitcoin_utxos.push(utxo);
                        }
                        Some(rune) => {
                            if rune.is_empty() {
                                bitcoin_utxos.push(utxo);
                            } else {
                                let rune = &rune[0];
                                let rune_id = RuneId::from_str(&rune.rune_id).unwrap();
                                write_utxo_manager(|manager| {
                                    manager.record_runic_utxos(
                                        addr,
                                        rune_id,
                                        vec![RunicUtxo {
                                            utxo,
                                            balance: rune.amount,
                                        }],
                                    );
                                });
                            }
                        }
                    }
                }
                write_utxo_manager(|manager| {
                    manager.record_bitcoin_utxos(addr, bitcoin_utxos);
                })
            }
        } */

        write_utxo_manager(|manager| manager.record_bitcoin_utxos(addr, utxo_response.utxos));

        match target {
            TargetType::Runic { ref runeid, target } => {
                let balance =
                    read_utxo_manager(|manager| manager.get_runestone_balance(addr, runeid));
                if balance < target && utxo_response.next_page.is_some() {
                    arg.filter = Some(UtxoFilter::Page(utxo_response.next_page.unwrap()));
                    continue;
                } else {
                    break;
                }
            }
            TargetType::Bitcoin { target } => {
                let balance = read_utxo_manager(|manager| manager.get_bitcoin_balance(addr));
                if balance < target && utxo_response.next_page.is_some() {
                    arg.filter = Some(UtxoFilter::Page(utxo_response.next_page.unwrap()));
                    continue;
                } else {
                    break;
                }
            }
        }
    }
}
