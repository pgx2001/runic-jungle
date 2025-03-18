use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{StableBTreeMap, Storable, storable::Bound};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use super::{
    CanisterMemory, CanisterMemoryIds, read_config, read_memory_manager, write_commission_state,
};

#[derive(CandidType, Deserialize)]
pub struct AgentDetail {
    pub agent_id: u128,
    pub created_at: u64,
    pub created_by: Principal,
    pub allocated_raw_subaccount: [u8; 32],
    pub txns: (Option<String>, Option<String>),
    pub runeid: Option<String>,
    pub name: String,
    pub ticker: u32,
    pub description: String,
    pub logo: Option<String>,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub openchat: Option<String>,
    pub discord: Option<String>,

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
    pub balances: HashSet<String>,
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
    pub fn logo_url(&self) -> Option<String> {
        if self.logo.is_none() {
            return None;
        }
        let localhost = read_config(|config| {
            config.bitcoin_network()
                == ic_cdk::api::management_canister::bitcoin::BitcoinNetwork::Regtest
        });
        let canister_id = ic_cdk::id();
        let url = if localhost {
            format!(
                "http://{canister_id}.raw.localhost:4943/agent/{}",
                self.agent_id
            )
        } else {
            format!("https://{canister_id}.raw.ic0.app/asset/{}", self.agent_id)
        };
        Some(url)
    }

    pub fn get_bitcoin_address(&self) -> String {
        crate::bitcoin::account_to_p2pkh_address(&icrc_ledger_types::icrc1::account::Account {
            owner: ic_cdk::id(),
            subaccount: Some(self.allocated_raw_subaccount),
        })
    }

    pub fn market_cap(&self) -> u128 {
        self.virtual_collateral_reserves
            .checked_mul(self.total_supply)
            .and_then(|result| result.checked_div(self.virtual_token_reserves))
            .unwrap_or(0)
    }

    pub fn agent_query(&self) -> crate::AgentDetails {
        crate::AgentDetails {
            created_at: self.created_at,
            created_by: self.created_by.to_text(),
            agent_name: self.name.clone(),
            logo: self.logo_url(),
            runeid: self.runeid.clone().unwrap_or(String::new()),
            ticker: self.ticker,
            description: self.description.clone(),
            website: self.website.clone(),
            twitter: self.twitter.clone(),
            openchat: self.openchat.clone(),
            discord: self.discord.clone(),
            total_supply: self.total_supply,
            holders: self.balances.len() as u32,
            market_cap: self.market_cap() as u64,
            current_prize_pool: self.current_prize_pool,
            current_winner: self.current_winner.clone(),
            txns: self.txns.clone(),
        }
    }

    /// Calculate the fees based on an input amount.
    /// Returns a tuple: (treasury_fee minus DEX fee, dex_fee).
    fn calculate_fee(&self, amount: u128) -> (u128, u128) {
        let treasury_fee = (amount * self.fee_bps as u128) / self.max_bps as u128;
        let dex_fee = (treasury_fee * self.dex_fee_bps as u128) / self.max_bps as u128;
        (treasury_fee - dex_fee, dex_fee)
    }

    /// Buy tokens with exact collateral in (BTC -> RUNE)
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

        // Record commission fees in balances (fees are in BTC, so update the bitcoin balance in the tuple)
        write_commission_state(|state| {
            let mut prev = state.get(&self.agent_id).unwrap_or(0);
            prev += treasury_fee as u64 + dex_fee as u64;
            state.insert(self.agent_id, prev);
        });
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

    pub fn buy_exact_out(
        &mut self,
        token_amount: u128,
        max_collateral: u128,
    ) -> Result<u128, &'static str> {
        // let commission_receiver = read_config(|config| config.commission_receiver());

        // Calculate collateral needed for token_amount
        let collateral_to_spend = token_amount
            .checked_mul(self.virtual_collateral_reserves)
            .ok_or("Calculation overflow")?
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

        // Record commission fees in balances (update BTC)
        write_commission_state(|state| {
            let mut prev = state.get(&self.agent_id).unwrap_or(0);
            prev += treasury_fee as u64 + dex_fee as u64;
            state.insert(self.agent_id, prev);
        });
        /* let entry = self
            .balances
            .entry(commission_receiver.clone())
            .or_insert((0, 0));
        entry.0 = entry
            .0
            .checked_add((treasury_fee + dex_fee) as u64)
            .ok_or("Commission transfer overflow")?; */

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

    /// Sell exact tokens in (RUNE -> BTC)
    pub fn sell_exact_in(
        &mut self,
        token_amount: u128,
        min_collateral_out: u128,
    ) -> Result<u128, &'static str> {
        // let commission_receiver = read_config(|config| config.commission_receiver());

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

        // Record commission fees in balances (update BTC)
        write_commission_state(|state| {
            let mut prev = state.get(&self.agent_id).unwrap_or(0);
            prev += treasury_fee as u64 + dex_fee as u64;
            state.insert(self.agent_id, prev);
        });
        /* let entry = self
            .balances
            .entry(commission_receiver.clone())
            .or_insert((0, 0));
        entry.0 = entry
            .0
            .checked_add((treasury_fee + dex_fee) as u64)
            .ok_or("Commission transfer overflow")?; */

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

    /// Sell tokens to receive exact collateral out (RUNE -> BTC)
    pub fn sell_exact_out(
        &mut self,
        max_token_amount: u128,
        collateral_out: u128,
    ) -> Result<u128, &'static str> {
        // let commission_receiver = read_config(|config| config.commission_receiver());

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

        // Record commission fees in balances (update BTC)
        write_commission_state(|state| {
            let mut prev = state.get(&self.agent_id).unwrap_or(0);
            prev += treasury_fee as u64 + dex_fee as u64;
            state.insert(self.agent_id, prev);
        });
        /* let entry = self
            .balances
            .entry(commission_receiver.clone())
            .or_insert((0, 0));
        entry.0 = entry
            .0
            .checked_add((treasury_fee + dex_fee) as u64)
            .ok_or("Commission transfer overflow")?; */

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
    pub fn get_agent_id(&mut self) -> u128 {
        let id = self._count;
        self._count += 1;
        id
    }

    pub fn find_agent_id(&self, agent_id: crate::AgentBy) -> Option<u128> {
        let id = match agent_id {
            crate::AgentBy::Id(id) => id,
            crate::AgentBy::Name(name) => match self._associated_set.get(&name) {
                None => return None,
                Some(id) => id,
            },
        };
        if self.mapping.get(&id).is_none() {
            None
        } else {
            Some(id)
        }
    }

    pub fn create_agent(
        &mut self,
        id: u128,
        created_by: Principal,
        allocated_raw_subaccount: [u8; 32],
        secret: String,
        name: String,
        ticker: u32,
        logo: Option<String>, // should be in uri format
        description: String,
        website: Option<String>,
        twitter: Option<String>,
        openchat: Option<String>,
        discord: Option<String>,
        total_supply: u128,
    ) -> (Option<String>, String) {
        let agent = AgentDetail {
            allocated_raw_subaccount,
            agent_id: id,
            created_at: ic_cdk::api::time(),
            created_by,
            txns: (None, None),
            runeid: None,
            name: name.clone(),
            ticker,
            logo,
            description,
            twitter,
            openchat,
            discord,
            website,

            total_supply,
            virtual_token_reserves: 750000000,
            virtual_collateral_reserves: 75000000,
            fee_bps: 30,
            dex_fee_bps: 3000,
            max_bps: 10000,

            secret: Some(secret),
            past_winners: HashSet::new(),
            current_winner: None,
            current_prize_pool: (0, 500_000_000),

            balances: HashSet::new(),
            rune: 1000_000_000,
            bitcoin: 0,
        };
        let url = agent.logo_url();
        let addr = agent.get_bitcoin_address();
        self._associated_set.insert(name, id);
        self.mapping.insert(id, agent);
        (url, addr)
    }

    pub fn delete_agent(&mut self, id: u128) {
        let agent = self.mapping.remove(&id);
        if let Some(agent) = agent {
            self._associated_set.remove(&agent.name);
        }
    }

    pub fn get_agents(&self) -> HashMap<u128, crate::AgentDetails> {
        let mut len = self.mapping.len() as u128 - 1;
        let mut map = HashMap::new();
        loop {
            let agent_query = match self.mapping.get(&len) {
                None => break,
                Some(agent) => agent.agent_query(),
            };
            map.insert(len, agent_query);
            len -= 1;
            if map.len() >= 50 {
                break;
            }
        }
        map
    }

    pub fn get_agent_of(&self, id: u128) -> Option<crate::AgentDetails> {
        self.mapping.get(&id).map(|detail| detail.agent_query())
    }
}
