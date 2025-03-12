use candid::{CandidType, Principal};
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
    pub commission: u16,
    pub commission_receiver: String,
    pub logo: String,
    pub project_description: String,
}

/// constructor for agent
// #[init]
pub fn init(
    InitArgs {
        commission,
        commission_receiver,
        logo,
        project_description,
    }: InitArgs,
) {
    let factory = ic_cdk::caller();
    write_config(|config| {
        let mut temp = config.get().clone();
        temp.commission = commission;
        temp.factory = factory;
        temp.commission_receiver = commission_receiver;
        temp.logo = logo;
        temp.project_description = project_description;
    })
}

/*
* NOTE: unimplemented for now
pub fn pre_upgrade() {}
pub fn post_upgrade() {}
*/

pub fn get_agent_bitcoin_address() -> String {
    todo!()
}

pub async fn initialize_rune_creation_process() {}

#[query]
pub fn generate_wallet() -> String {
    let caller = ic_cdk::caller();
    todo!()
}

pub fn get_balances() {}

pub fn market_cap() {}

/*
* amount: no of tokens to be bought
*/
pub fn buy(amount: u128) {
    let caller = ic_cdk::caller();
}

/*
* amount: no of tokens to be sold
*/
pub fn sell(amount: u128) {
    let caller = ic_cdk::caller();
}

#[derive(CandidType)]
pub struct PrizePool {
    pub bitcoin: u64,
    pub rune: (String, u128),
}

#[query]
pub fn current_prize_pool() -> PrizePool {
    // TODO: fetch runeid from config
    read_prize_pool(|pool| pool.query_prize_pool(String::from("")))
}

#[update(guard = "trap_if_anonymous")]
pub async fn try_withdraw(msg: String) -> String {
    let caller = ic_cdk::caller();
    // let mut llm = ICLLM::new(false);
    todo!()
}

#[update]
pub async fn chat(message: String) -> String {
    let caller = ic_cdk::caller();
    let mut llm = ICLLM::chatbot();
    llm.chat(message).await
}

pub fn http_request() {}

/// this function will be called within a fixed interval of time
async fn sync() {}

ic_cdk::export_candid!();
