use std::collections::{HashMap, HashSet};

use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{StableBTreeMap, Storable, storable::Bound};
use serde::Deserialize;

use super::{CanisterMemory, CanisterMemoryIds, read_memory_manager};

#[derive(CandidType, Deserialize)]
pub struct AgentDetail {
    pub agent_id: u128,
    pub created_at: u64,
    pub allocated_raw_subaccount: [u8; 32],
    pub txns: (Option<String>, Option<String>),
    pub runeid: Option<String>,
    pub name: String,
    pub ticker: u32,

    // bait the bot
    pub past_winners: HashSet<(u64, u64, u128, Principal, String)>, // data -> (time, amount_in_bitcoin, amount_in_rune, winner, secret)
    pub current_prize_pool: (u64, u128),
    pub secret: Option<String>,
    pub current_winner: Option<Principal>,

    // market maker
    pub total_supply: u128,
    pub virtual_token_reserves: u128,      // Virtual RUNE reserves
    pub virtual_collateral_reserves: u128, // Virtual BTC reserves
    pub fee_bps: u16,                      // Fee in basis points
    pub dex_fee_bps: u16,                  // DEX fee in basis points
    pub max_bps: u16,                      // Maximum basis points (typically 10000 for 100%)

    // balances
    pub bitcoin: u128,
    pub rune: u128,

    // user balances
    pub balances: HashMap<String, (u64, u128)>,
}

impl Storable for AgentDetail {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).expect("should encode"))
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("should decode")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl AgentDetail {
    // Calculate the market cap
    pub fn market_cap(&self) -> u128 {
        let mc = self
            .virtual_collateral_reserves
            .checked_mul(10u128.pow(18))
            .and_then(|result| result.checked_mul(self.total_supply))
            .and_then(|result| result.checked_div(self.virtual_token_reserves))
            .unwrap_or(0);

        mc.checked_div(10u128.pow(18)).unwrap_or(0)
    }

    // Calculate fees
    fn calculate_fee(&self, amount: u128) -> (u128, u128) {
        let treasury_fee = (amount * self.fee_bps as u128) / self.max_bps as u128;
        let dex_fee = (treasury_fee * self.dex_fee_bps as u128) / self.max_bps as u128;
        (treasury_fee - dex_fee, dex_fee)
    }

    // Buy tokens with exact collateral in (BTC -> RUNE)
    pub fn buy_exact_in(
        &mut self,
        collateral_in: u128,
        min_tokens_out: u128,
    ) -> Result<u128, &'static str> {
        // Calculate fees
        let (treasury_fee, dex_fee) = self.calculate_fee(collateral_in);
        let collateral_to_spend = collateral_in
            .checked_sub(treasury_fee)
            .ok_or("Fee subtraction underflow")?
            .checked_sub(dex_fee)
            .ok_or("Fee subtraction underflow")?;

        // Calculate tokens to receive
        let tokens_out = (collateral_to_spend * self.virtual_token_reserves)
            .checked_div(
                self.virtual_collateral_reserves
                    .checked_add(collateral_to_spend)
                    .ok_or("Collateral addition overflow")?,
            )
            .ok_or("Calculation overflow")?;

        // Slippage check
        if tokens_out < min_tokens_out {
            return Err("Slippage check failed");
        }

        // Update virtual reserves
        self.virtual_token_reserves = self
            .virtual_token_reserves
            .checked_sub(tokens_out)
            .ok_or("Insufficient token reserves")?;
        self.virtual_collateral_reserves = self
            .virtual_collateral_reserves
            .checked_add(collateral_to_spend)
            .ok_or("Collateral reserve overflow")?;

        // Update actual balances
        self.bitcoin = self
            .bitcoin
            .checked_add(collateral_in)
            .ok_or("Bitcoin balance overflow")?;
        self.rune = self
            .rune
            .checked_sub(tokens_out)
            .ok_or("Insufficient rune balance")?;

        Ok(tokens_out)
    }

    // Buy exact tokens out (BTC -> RUNE)
    pub fn buy_exact_out(
        &mut self,
        token_amount: u128,
        max_collateral: u128,
    ) -> Result<u128, &'static str> {
        // Calculate collateral needed
        let collateral_to_spend = (token_amount
            .checked_mul(self.virtual_collateral_reserves)
            .ok_or("Calculation overflow")?)
        .checked_div(
            self.virtual_token_reserves
                .checked_sub(token_amount)
                .ok_or("Token amount exceeds reserves")?,
        )
        .ok_or("Division by zero")?;

        // Calculate fees
        let (treasury_fee, dex_fee) = self.calculate_fee(collateral_to_spend);
        let collateral_with_fee = collateral_to_spend
            .checked_add(treasury_fee)
            .and_then(|sum| sum.checked_add(dex_fee))
            .ok_or("Fee calculation overflow")?;

        // Slippage check
        if collateral_with_fee > max_collateral {
            return Err("Slippage check failed");
        }

        // Update virtual reserves
        self.virtual_token_reserves = self
            .virtual_token_reserves
            .checked_sub(token_amount)
            .ok_or("Insufficient token reserves")?;
        self.virtual_collateral_reserves = self
            .virtual_collateral_reserves
            .checked_add(collateral_to_spend)
            .ok_or("Collateral reserve overflow")?;

        // Update actual balances
        self.bitcoin = self
            .bitcoin
            .checked_add(collateral_with_fee)
            .ok_or("Bitcoin balance overflow")?;
        self.rune = self
            .rune
            .checked_sub(token_amount)
            .ok_or("Insufficient rune balance")?;

        Ok(collateral_with_fee)
    }

    // Sell exact tokens in (RUNE -> BTC)
    pub fn sell_exact_in(
        &mut self,
        token_amount: u128,
        min_collateral_out: u128,
    ) -> Result<u128, &'static str> {
        // Calculate collateral to receive
        let collateral_to_receive = (token_amount
            .checked_mul(self.virtual_collateral_reserves)
            .ok_or("Calculation overflow")?)
        .checked_div(
            self.virtual_token_reserves
                .checked_add(token_amount)
                .ok_or("Token addition overflow")?,
        )
        .ok_or("Division by zero")?;

        // Calculate fees
        let (treasury_fee, dex_fee) = self.calculate_fee(collateral_to_receive);
        let collateral_minus_fee = collateral_to_receive
            .checked_sub(treasury_fee)
            .and_then(|diff| diff.checked_sub(dex_fee))
            .ok_or("Fee subtraction underflow")?;

        // Slippage check
        if collateral_minus_fee < min_collateral_out {
            return Err("Slippage check failed");
        }

        // Update virtual reserves
        self.virtual_token_reserves = self
            .virtual_token_reserves
            .checked_add(token_amount)
            .ok_or("Token reserve overflow")?;
        self.virtual_collateral_reserves = self
            .virtual_collateral_reserves
            .checked_sub(collateral_to_receive)
            .ok_or("Insufficient collateral reserves")?;

        // Update actual balances
        self.bitcoin = self
            .bitcoin
            .checked_sub(collateral_minus_fee)
            .ok_or("Insufficient bitcoin balance")?;
        self.rune = self
            .rune
            .checked_add(token_amount)
            .ok_or("Rune balance overflow")?;

        Ok(collateral_minus_fee)
    }

    // Sell tokens to receive exact collateral out (RUNE -> BTC)
    pub fn sell_exact_out(
        &mut self,
        max_token_amount: u128,
        collateral_out: u128,
    ) -> Result<u128, &'static str> {
        // Calculate fees
        let (treasury_fee, dex_fee) = self.calculate_fee(collateral_out);
        let total_collateral_needed = collateral_out
            .checked_add(treasury_fee)
            .and_then(|sum| sum.checked_add(dex_fee))
            .ok_or("Fee addition overflow")?;

        // Calculate tokens needed
        let tokens_needed = (total_collateral_needed
            .checked_mul(self.virtual_token_reserves)
            .ok_or("Calculation overflow")?)
        .checked_div(
            self.virtual_collateral_reserves
                .checked_sub(total_collateral_needed)
                .ok_or("Collateral subtraction underflow")?,
        )
        .ok_or("Division by zero")?;

        // Slippage check
        if tokens_needed > max_token_amount {
            return Err("Slippage check failed");
        }

        // Update virtual reserves
        self.virtual_token_reserves = self
            .virtual_token_reserves
            .checked_add(tokens_needed)
            .ok_or("Token reserve overflow")?;
        self.virtual_collateral_reserves = self
            .virtual_collateral_reserves
            .checked_sub(total_collateral_needed)
            .ok_or("Insufficient collateral reserves")?;

        // Update actual balances
        self.bitcoin = self
            .bitcoin
            .checked_sub(collateral_out)
            .ok_or("Insufficient bitcoin balance")?;
        self.rune = self
            .rune
            .checked_add(tokens_needed)
            .ok_or("Rune balance overflow")?;

        Ok(tokens_needed)
    }

    // Utility functions for calculating amounts and fees

    // Get amount out and fee for a given input
    pub fn get_amount_out_and_fee(
        &self,
        amount_in: u128,
        reserve_in: u128,
        reserve_out: u128,
        payment_token_is_in: bool,
    ) -> Result<(u128, u128), &'static str> {
        if payment_token_is_in {
            // Calculate fee first, then amount out
            let (treasury_fee, dex_fee) = self.calculate_fee(amount_in);
            let fee = treasury_fee
                .checked_add(dex_fee)
                .ok_or("Fee addition overflow")?;

            let amount_minus_fee = amount_in
                .checked_sub(fee)
                .ok_or("Fee subtraction underflow")?;
            let amount_out = amount_minus_fee
                .checked_mul(reserve_out)
                .and_then(|result| result.checked_div(reserve_in.checked_add(amount_minus_fee)?))
                .ok_or("Calculation error")?;

            Ok((amount_out, fee))
        } else {
            // Calculate amount out first, then fee
            let amount_out = amount_in
                .checked_mul(reserve_out)
                .and_then(|result| result.checked_div(reserve_in.checked_add(amount_in)?))
                .ok_or("Calculation error")?;

            let (treasury_fee, dex_fee) = self.calculate_fee(amount_out);
            let fee = treasury_fee
                .checked_add(dex_fee)
                .ok_or("Fee addition overflow")?;

            Ok((amount_out, fee))
        }
    }

    // Get amount in and fee for a given output
    pub fn get_amount_in_and_fee(
        &self,
        amount_out: u128,
        reserve_in: u128,
        reserve_out: u128,
        payment_token_is_out: bool,
    ) -> Result<(u128, u128), &'static str> {
        if payment_token_is_out {
            // Calculate fee first, then amount in
            let (treasury_fee, dex_fee) = self.calculate_fee(amount_out);
            let fee = treasury_fee
                .checked_add(dex_fee)
                .ok_or("Fee addition overflow")?;

            let total_out = amount_out.checked_add(fee).ok_or("Fee addition overflow")?;
            let amount_in = total_out
                .checked_mul(reserve_in)
                .and_then(|result| result.checked_div(reserve_out.checked_sub(total_out)?))
                .ok_or("Calculation error")?;

            Ok((amount_in, fee))
        } else {
            // Calculate amount in first, then fee
            let amount_in = amount_out
                .checked_mul(reserve_in)
                .and_then(|result| result.checked_div(reserve_out.checked_sub(amount_out)?))
                .ok_or("Calculation error")?;

            let (treasury_fee, dex_fee) = self.calculate_fee(amount_in);
            let fee = treasury_fee
                .checked_add(dex_fee)
                .ok_or("Fee addition overflow")?;

            Ok((amount_in, fee))
        }
    }

    pub fn buy(&mut self, collateral_in: u128, min_tokens_out: u128) -> Result<u128, &'static str> {
        self.buy_exact_in(collateral_in, min_tokens_out)
    }

    pub fn sell(
        &mut self,
        token_amount: u128,
        min_collateral_out: u128,
    ) -> Result<u128, &'static str> {
        self.sell_exact_in(token_amount, min_collateral_out)
    }
}

pub type AgentMapping = StableBTreeMap<u128, AgentDetail, CanisterMemory>;

pub type AssociatedAgentSet = StableBTreeMap<String, u128, CanisterMemory>;

pub struct AgentState {
    pub mapping: AgentMapping,
    _count: u128,
    _associated_set: AssociatedAgentSet,
}

impl Default for AgentState {
    fn default() -> Self {
        read_memory_manager(|manager| Self {
            mapping: AgentMapping::init(manager.get(CanisterMemoryIds::Agent.into())),
            _count: 0,
            _associated_set: AssociatedAgentSet::init(
                manager.get(CanisterMemoryIds::AssociatedAgentSet.into()),
            ),
        })
    }
}

impl AgentState {
    fn get_agent_id(&mut self) -> u128 {
        let id = self._count;
        self._count += 1;
        id
    }

    pub fn create_agent(&mut self) -> u128 {
        let id = self.get_agent_id();
        todo!()
    }

    pub fn get_agents(&self) -> HashSet<u128, ()> {
        todo!()
    }

    pub fn get_agent_of(&self) {}

    pub fn get_amount_out_and_fee(&self) {}

    pub fn buy(&mut self) {}

    pub fn sell(&mut self) {}

    pub fn get_lucky_draw_detail(&self) {}

    pub fn bait_the_bot(&mut self) {}
}
