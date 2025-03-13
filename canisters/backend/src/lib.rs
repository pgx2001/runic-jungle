// modules
mod state;

use candid::CandidType;
// re export
use ic_cdk::api::management_canister::ecdsa::EcdsaPublicKeyResponse as EcdsaPublicKey;
use serde::Deserialize;

pub fn init() {
    let caller = ic_cdk::caller();
}

/* canister upgrade hooks
pub fn pre_upgrade() {}
pub fn post_upgrade() {}
*/

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

pub fn create_agent(
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
}

#[derive(CandidType, Deserialize)]
pub enum AgentBy {
    Id(u128),
    Name(String),
}

pub struct BuyArgs {
    pub id: AgentBy,
}

pub fn buy() {}

pub struct SellArgs {
    pub id: AgentBy,
}

pub fn sell() {}

pub struct LuckyDraw {
    pub id: AgentBy,
}

pub fn bait_the_bot() {}

pub struct ChatArgs {
    pub agent: AgentBy,
    pub session_id: u128,
}

pub fn chat() {}

pub fn http_request() {}

ic_cdk::export_candid!();
