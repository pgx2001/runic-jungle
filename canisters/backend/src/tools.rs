use crate::{read_config, read_ledger_entries};
use candid::Principal;
use ic_cdk::api::management_canister::bitcoin::{GetBalanceRequest, bitcoin_get_balance};

pub async fn get_bitcoin_balance(address: String) -> u64 {
    let network = read_config(|config| config.bitcoin_network());
    bitcoin_get_balance(GetBalanceRequest {
        address,
        network,
        min_confirmations: None,
    })
    .await
    .expect("should fetch the balance")
    .0
}

pub fn get_rune_balance(agent: &u128, user: &Principal) -> u128 {
    read_ledger_entries(|entries| {
        let entry = entries.get(user).unwrap_or_default();
        entry
            .ledger_entries
            .get(agent)
            .copied()
            .unwrap_or_default()
            .1
    })
}
