use candid::CandidType;
use ic_cdk::{init, query, update};
use llm::ICLLM;
use serde::Deserialize;

mod chains;
mod guard;
mod indexer;
mod llm;
mod state;

use guard::*;
use state::*;

use ic_cdk::api::management_canister::ecdsa::EcdsaPublicKeyResponse as EcdsaPublicKey;

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub commission_receiver: String,
    pub runeid: String,
    pub logo: String,
    pub project_description: String,
}

/// constructor for agent
#[init]
pub fn init(
    InitArgs {
        commission_receiver,
        runeid,
        logo,
        project_description,
    }: InitArgs,
) {
    let factory = ic_cdk::caller();
    write_config(|config| {})
}

/*
* NOTE: unimplemented for now
pub fn pre_upgrade() {}
pub fn post_upgrade() {}
*/

#[query]
pub fn generate_wallet() -> String {
    let caller = ic_cdk::caller();
    todo!()
}

pub fn market_cap() {}

/*
* amount: no of tokens to be bought
*/
pub fn buy(amount: u128) {}

/*
* amount: no of tokens to be sold
*/
pub fn sell(amount: u128) {}

#[derive(CandidType)]
pub struct PrizePool {
    pub bitcoin: u64,
    pub rune: (String, u128),
}

pub fn current_prize_pool() -> PrizePool {
    todo!()
}

#[update(guard = "trap_if_anonymous")]
pub async fn try_withdraw(msg: String) {
    let caller = ic_cdk::caller();
}

#[derive(CandidType, Deserialize)]
pub enum ChatType {
    Anonymous,
    LoggedIn,
}

#[update]
pub async fn chat(message: String) -> String {
    let mut llm = ICLLM::new(true);
    llm.chat(message).await
}

pub fn http_request() {}

/// this function will be called within a fixed interval of time
async fn sync() {}

ic_cdk::export_candid!();
