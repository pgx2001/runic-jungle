// modules
mod bitcoin;
mod indexer;
mod llm;
mod state;
mod tools;

use std::collections::HashMap;

use candid::{CandidType, define_function};

// re export
use ::bitcoin as bitcoin_lib;
use ic_cdk::api::management_canister::ecdsa::EcdsaPublicKeyResponse as EcdsaPublicKey;
use ic_cdk::{init, query, update};
use serde::Deserialize;

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
}

/* canister upgrade hooks
pub fn pre_upgrade() {}
pub fn post_upgrade() {}
*/

pub fn get_agents() -> HashMap<u128, ()> {
    todo!()
}

pub fn get_agent_of(id: AgentBy) -> Option<()> {
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
) {
    let caller = ic_cdk::caller();
}

#[derive(CandidType, Deserialize)]
pub enum AgentBy {
    Id(u128),
    Name(String),
}

#[derive(CandidType, Deserialize)]
pub struct BuyArgs {
    pub id: AgentBy,
}

pub fn buy() {
    let caller = ic_cdk::caller();
}

#[derive(CandidType, Deserialize)]
pub struct SellArgs {
    pub id: AgentBy,
}

pub fn sell() {
    let caller = ic_cdk::caller();
}

#[derive(CandidType)]
pub struct LuckyDraw {
    pub id: AgentBy,
    pub message: String,
}

pub fn lucky_draw(LuckyDraw { id, message }: LuckyDraw) {
    let caller = ic_cdk::caller();
}

pub struct ChatArgs {
    pub agent: AgentBy,
    pub session_id: u128,
    pub message: String,
}

pub fn chat(
    ChatArgs {
        agent,
        session_id,
        message,
    }: ChatArgs,
) {
    let caller = ic_cdk::caller();
}

#[derive(CandidType, Deserialize, Clone)]
pub struct HeaderField(pub String, pub String);

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

pub fn http_request(req: HttpRequest) -> HttpResponse {
    todo!()
}

ic_cdk::export_candid!();
