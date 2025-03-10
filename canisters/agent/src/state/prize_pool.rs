use std::collections::HashMap;

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{Storable, storable::Bound};
use serde::Deserialize;

#[derive(CandidType, Deserialize, Default)]
pub struct PrizePool {
    // mapping of round to (timestamp, winner's address, amount, txid)
    pub past_winners: HashMap<u128, (u64, String, u128, Option<String>)>,
    pub magical_words: String,
    pub bitcoin: u64,
    pub rune: u128,
}

impl Storable for PrizePool {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).expect("should encode"))
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("should decode")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl PrizePool {
    pub fn query_prize_pool(&self, runeid: String) -> crate::PrizePool {
        crate::PrizePool {
            bitcoin: self.bitcoin,
            rune: (runeid, self.rune),
        }
    }
}
