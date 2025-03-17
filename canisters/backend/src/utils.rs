use candid::Principal;
use icrc_ledger_types::icrc1::account::Account;
use tiny_keccak::{Hasher, Sha3};

pub fn principal_to_subaccount(principal: &Principal) -> [u8; 32] {
    let mut hash = [0u8; 32];
    let mut hasher = Sha3::v256();
    hasher.update(principal.as_slice());
    hasher.finalize(&mut hash);
    hash
}

pub fn get_account_for(principal: &Principal) -> Account {
    let subaccount = principal_to_subaccount(principal);
    Account {
        owner: ic_cdk::id(),
        subaccount: Some(subaccount),
    }
}

pub fn generate_subaccount_for_agent(id: u128) -> [u8; 32] {
    let mut hash = [0u8; 32];
    let mut hasher = Sha3::v256();
    hasher.update(ic_cdk::id().as_slice());
    hasher.update(id.to_le_bytes().as_ref());
    hasher.finalize(&mut hash);
    hash
}
