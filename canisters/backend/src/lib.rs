// modules
mod bitcoin;
mod indexer;
mod llm;
mod state;
mod tools;
mod utils;

use state::*;
use std::collections::HashMap;

use candid::{CandidType, define_function};

// re export
use ::bitcoin as bitcoin_lib;
use ic_cdk::api::management_canister::ecdsa::EcdsaPublicKeyResponse as EcdsaPublicKey;
use ic_cdk::api::management_canister::ecdsa::{
    EcdsaKeyId, EcdsaPublicKeyArgument, ecdsa_public_key,
};
use ic_cdk::{init, query, update};
use serde::Deserialize;

async fn lazy_ecdsa_setup() {
    let ecdsa_keyid: EcdsaKeyId = read_config(|config| config.ecdsakeyid());
    let ecdsa_response = ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: vec![],
        key_id: ecdsa_keyid,
    })
    .await
    .expect("Failed to get ecdsa key")
    .0;

    write_config(|config| {
        let mut temp = config.get().clone();
        temp.ecdsa_public_key = Some(ecdsa_response);
        let _ = config.set(temp);
    });
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub creation_fee: u64,
    pub commission: u16,
    pub commission_receiver: String,
}

pub fn init(
    InitArgs {
        creation_fee,
        commission,
        commission_receiver,
    }: InitArgs,
) {
    let caller = ic_cdk::caller();
    // TODO: config initialization
    ic_cdk_timers::set_timer(std::time::Duration::from_secs(5), || {
        ic_cdk::spawn(lazy_ecdsa_setup())
    });
}

/* canister upgrade hooks
pub fn pre_upgrade() {}
pub fn post_upgrade() {}
*/

#[query]
pub fn get_deposit_address() -> String {
    let caller = ic_cdk::caller();
    let account = utils::get_account_for(&caller);
    bitcoin::account_to_p2pkh_address(&account)
}

#[derive(CandidType, Deserialize)]
pub enum WithdrawalType {
    Bitcoin { amount: u64 },
    Rune { runeid: AgentBy, amount: u128 },
}

#[update]
pub async fn withdraw(to: String, withdrawal_type: WithdrawalType) -> u128 {
    todo!()
}

#[derive(CandidType)]
pub struct AgentDetails {
    pub created_at: u64,
    pub created_by: String,
    pub agent_name: String,
    pub logo: Option<String>,
    pub runeid: String,
    pub ticker: u32,
    pub description: String,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub openchat: Option<String>,
    pub discord: Option<String>,
    pub total_supply: u128,
    pub holders: u32,
    pub market_cap: u64,
    pub current_prize_pool: (u64, u128),
    pub txns: (Option<String>, Option<String>),
}

#[query]
pub fn get_agents() -> HashMap<u128, AgentDetails> {
    todo!()
}

#[query]
pub fn get_agent_of(id: AgentBy) -> Option<AgentDetails> {
    todo!()
}

#[derive(CandidType, Deserialize)]
pub struct CreateAgentArgs {
    pub name: String,
    pub ticker: Option<u32>,
    pub logo: Option<String>, // should be in uri format
    pub description: String,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub openchat: Option<String>,
    pub discord: Option<String>,
}

#[update]
pub async fn create_agent(
    CreateAgentArgs {
        name,
        ticker,
        logo,
        description,
        website,
        twitter,
        openchat,
        discord,
    }: CreateAgentArgs,
) -> u128 {
    let caller = ic_cdk::caller();
    let account = utils::get_account_for(&caller);
    let bitcoin_address = bitcoin::account_to_p2pkh_address(&account);

    // TODO: get the balance
    // TODO: calculate the fee
    // TODO: Store the agent
    // TODO: start the submission timer
    todo!()
}

#[derive(CandidType, Deserialize)]
pub enum AgentBy {
    Id(u128),
    Name(String),
}

#[derive(CandidType, Deserialize)]
pub struct BuyArgs {
    pub id: AgentBy,
    pub min_amount_out: u128,
}

#[update]
pub fn buy(BuyArgs { id, min_amount_out }: BuyArgs) {
    let caller = ic_cdk::caller();
    let account = utils::get_account_for(&caller);
    let bitcoin_address = bitcoin::account_to_p2pkh_address(&account);
}

#[derive(CandidType, Deserialize)]
pub struct SellArgs {
    pub id: AgentBy,
    pub min_amount_out: u64,
}

#[update]
pub fn sell(SellArgs { id, min_amount_out }: SellArgs) {
    let caller = ic_cdk::caller();
    let account = utils::get_account_for(&caller);
    let bitcoin_address = bitcoin::account_to_p2pkh_address(&account);
}

#[derive(CandidType, Deserialize)]
pub struct LuckyDraw {
    pub id: AgentBy,
    pub message: String,
}

#[update]
pub fn lucky_draw(LuckyDraw { id, message }: LuckyDraw) {
    let caller = ic_cdk::caller();
    let account = utils::get_account_for(&caller);
    let bitcoin_address = bitcoin::account_to_p2pkh_address(&account);
}

#[derive(CandidType, Deserialize)]
pub struct ChatArgs {
    pub agent: AgentBy,
    pub session_id: Option<u128>,
    pub message: String,
}

#[update]
pub fn chat(
    ChatArgs {
        agent,
        session_id,
        message,
    }: ChatArgs,
) {
    let caller = ic_cdk::caller();
    let session_id = session_id
        .unwrap_or_else(|| write_chat_session(|session| session.start_new_session(caller.clone())));
    todo!()
}

#[derive(CandidType, Deserialize, Clone)]
pub struct HeaderField(pub String, pub String);

#[derive(CandidType, Deserialize, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: Vec<u8>,
    pub streaming_strategy: Option<StreamingStrategy>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CreateStrategyArgs {
    pub asset_id: u128,
    pub chunk_index: u32,
    pub chunk_size: u32,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct StreamingCallbackToken {
    pub asset_id: u128,
    pub chunk_index: u32,
    pub chunk_size: u32,
    pub content_encoding: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub enum StreamingStrategy {
    Callback {
        token: StreamingCallbackToken,
        callback: CallbackFunc,
    },
}

#[derive(CandidType, Deserialize, Clone)]
pub struct StreamingCallbackHttpResponse {
    pub body: Vec<u8>,
    pub token: Option<StreamingCallbackToken>,
}

define_function!(pub CallbackFunc: () -> () query);

#[query]
pub fn http_request(req: HttpRequest) -> HttpResponse {
    todo!()
}

ic_cdk::export_candid!();
