mod state;
mod chain;

pub fn init() {}

pub fn pre_upgrade(){}

pub fn post_upgrade(){}

/// creates token and agent canister
pub async fn create_agent(){}

ic_cdk::export_candid!();
