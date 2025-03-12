use candid::{CandidType, Deserialize};

mod chain;
mod state;

pub fn init() {}

pub fn pre_upgrade() {}

pub fn post_upgrade() {}

#[derive(CandidType, Deserialize)]
pub struct CreateAgentArgs {
    pub name: String,
    pub logo: String,
    pub ticker: Option<u32>,
    pub description: String,
}

/// creates token and agent canister
pub async fn create_agent() {}

ic_cdk::export_candid!();
