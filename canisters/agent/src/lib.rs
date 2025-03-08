/// constructor for agent
pub fn init() {
    let caller = ic_cdk::caller();
}

pub fn pre_upgrade() {}

pub fn post_upgrade() {}

ic_cdk::export_candid!();
