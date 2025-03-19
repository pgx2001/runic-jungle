// modules
mod bitcoin;
mod indexer;
mod llm;
mod state;
mod tools;
mod txn_handler;
mod utils;

use bitcoin::runestone::etch::EtchingArgs;
use state::*;
use std::collections::HashMap;

use candid::{CandidType, define_function};

// re export
use ::bitcoin as bitcoin_lib;
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;
use ic_cdk::api::management_canister::ecdsa::EcdsaPublicKeyResponse as EcdsaPublicKey;
use ic_cdk::api::management_canister::ecdsa::{EcdsaPublicKeyArgument, ecdsa_public_key};
use ic_cdk::api::management_canister::schnorr::{
    SchnorrPublicKeyArgument, SchnorrPublicKeyResponse as SchnorrPublicKey, schnorr_public_key,
};
use ic_cdk::{init, query, update};
use serde::Deserialize;

async fn lazy_ecdsa_schnorr_setup() {
    let (ecdsakeyid, schnorrkeyid) =
        read_config(|config| (config.ecdsakeyid(), config.schnorrkeyid()));
    let ecdsapublickey = ecdsa_public_key(EcdsaPublicKeyArgument {
        derivation_path: vec![],
        canister_id: None,
        key_id: ecdsakeyid,
    })
    .await
    .unwrap()
    .0;
    let schnorrpublickey = schnorr_public_key(SchnorrPublicKeyArgument {
        derivation_path: vec![],
        canister_id: None,
        key_id: schnorrkeyid,
    })
    .await
    .unwrap()
    .0;

    write_config(|config| {
        let mut temp = config.get().clone();
        temp.ecdsa_public_key.replace(ecdsapublickey);
        temp.schnorr_public_key.replace(schnorrpublickey);
        config.set(temp).expect("failed to set config");
    });
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub bitcoin_network: BitcoinNetwork,
    pub commission_receiver: Option<candid::Principal>,
}

#[init]
pub fn init(
    InitArgs {
        bitcoin_network,
        commission_receiver,
    }: InitArgs,
) {
    let caller = ic_cdk::caller();
    let commission_receiver = commission_receiver.unwrap_or(caller);
    let keyname = match bitcoin_network {
        BitcoinNetwork::Mainnet => "key_1".to_string(),
        BitcoinNetwork::Testnet => "test_key_1".to_string(),
        BitcoinNetwork::Regtest => "dfx_test_key".to_string(),
    };
    let max_allowed_agent = if bitcoin_network != BitcoinNetwork::Regtest {
        10
    } else {
        100
    };
    write_config(|config| {
        let mut temp = config.get().clone();
        temp.keyname = keyname;
        temp.bitcoin_network = bitcoin_network;
        temp.commission_receiver = commission_receiver;
        temp.allowed_agent_count = max_allowed_agent;
        config.set(temp).expect("failed to set config");
    });
    ic_cdk_timers::set_timer(std::time::Duration::from_secs(5), || {
        ic_cdk::spawn(lazy_ecdsa_schnorr_setup())
    });
}

/* canister upgrade hooks
pub fn pre_upgrade() {}
pub fn post_upgrade() {}
*/

#[update]
pub fn increase_allowed_agent_count(by: u128) {
    let caller = ic_cdk::caller();
    write_config(|config| {
        let mut temp = config.get().clone();
        if temp.auth != Some(caller) {
            ic_cdk::trap("Unauthorized")
        }
        temp.allowed_agent_count += by;
        let _ = config.set(temp);
    })
}

#[query]
pub fn get_deposit_address() -> String {
    let caller = ic_cdk::caller();
    let account = utils::get_account_for(&caller);
    bitcoin::account_to_p2pkh_address(&account)
}

#[update]
pub async fn get_bitcoin_balance() -> u64 {
    let caller = ic_cdk::caller();
    let account = utils::get_account_for(&caller);
    let bitcoin_address = bitcoin::account_to_p2pkh_address(&account);
    let bitcoin_balance = ic_cdk::api::management_canister::bitcoin::bitcoin_get_balance(
        ic_cdk::api::management_canister::bitcoin::GetBalanceRequest {
            address: bitcoin_address,
            network: read_config(|config| config.bitcoin_network()),
            min_confirmations: None,
        },
    )
    .await
    .unwrap()
    .0;
    read_ledger_entries(|entries| {
        let entry = entries.get(&caller).unwrap_or_default();
        bitcoin_balance - entry.restricted_bitcoin_balance
    })
}

#[update]
pub async fn get_balances() -> HashMap<String, u128> {
    let caller = ic_cdk::caller();
    let account = utils::get_account_for(&caller);
    let bitcoin_address = bitcoin::account_to_p2pkh_address(&account);
    let bitcoin_balance = ic_cdk::api::management_canister::bitcoin::bitcoin_get_balance(
        ic_cdk::api::management_canister::bitcoin::GetBalanceRequest {
            address: bitcoin_address,
            network: read_config(|config| config.bitcoin_network()),
            min_confirmations: None,
        },
    )
    .await
    .unwrap()
    .0;
    read_ledger_entries(|entries| {
        let entry = entries.get(&caller).unwrap_or_default();
        let mut map = HashMap::new();
        map.insert(
            String::from("Bitcoin"),
            bitcoin_balance as u128 - entry.restricted_bitcoin_balance as u128,
        );
        for (rune, (_, balance)) in entry.ledger_entries {
            let rune = read_agents(|agents| agents.mapping.get(&rune).unwrap().name);
            map.insert(rune, balance);
        }
        map
    })
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
    pub current_winner: Option<candid::Principal>,
    pub txns: (Option<String>, Option<String>),
}

#[query]
pub fn get_agents() -> HashMap<u128, AgentDetails> {
    read_agents(|agents| agents.get_agents())
}

#[query]
pub fn get_agent_of(id: AgentBy) -> Option<AgentDetails> {
    read_agents(|agents| {
        let id = agents.find_agent_id(id)?;
        agents.get_agent_of(id)
    })
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

    //get the balance
    indexer::fetch_utxos_and_update(
        &bitcoin_address,
        indexer::TargetType::Bitcoin { target: u64::MAX },
    )
    .await;

    let bitcoin_balance = read_ledger_entries(|entries| {
        let entry = entries.get(&caller).unwrap_or_default();
        let bitcoin_balance =
            read_utxo_manager(|manager| manager.get_bitcoin_balance(&bitcoin_address));
        bitcoin_balance - entry.restricted_bitcoin_balance
    });

    ic_cdk::println!("{}", bitcoin_balance);

    if bitcoin_balance < 30_000 {
        ic_cdk::trap("not enough balance")
    }

    let (spaced_rune, total_supply, symbol) =
        bitcoin::runestone::validate_etching(&name, ticker, 3, 1_000_000)
            .expect("Etching Arg validation failed");

    // checking if rune is occupied
    if indexer::runes_indexer::get_rune(spaced_rune.to_string())
        .await
        .is_some()
    {
        ic_cdk::trap("Rune already taken")
    }

    let secret = llm::Llm::generate_secret_word(&spaced_rune.to_string(), &description).await;
    ic_cdk::println!("secret word: {}", secret);

    let (id, (logo_url, agent_address)) = write_agents(|agents| {
        let id = agents.get_agent_id();
        let allocated_raw_subaccount = utils::generate_subaccount_for_agent(id);
        let resp = agents.create_agent(
            id,
            caller,
            allocated_raw_subaccount,
            secret,
            spaced_rune.to_string(),
            symbol.unwrap_or('•') as u32,
            logo,
            description,
            website,
            twitter,
            openchat,
            discord,
            total_supply,
        );
        (id, resp)
    });

    let (content_type, logo) = match logo_url {
        None => (None, None),
        Some(logo) => (
            Some(String::from("text/html").as_bytes().to_vec()),
            Some(format!("<img src=\"{}\" />", logo).as_bytes().to_vec()),
        ),
    };

    let fee_payer = bitcoin::address_validation(&bitcoin_address).unwrap();
    let agent_address = bitcoin::address_validation(&agent_address).unwrap();

    let fee_per_vbytes = bitcoin::get_fee_per_vbyte().await;

    let handler = match bitcoin::runestone::etch::etch(EtchingArgs {
        agent_id: id,
        reveal_address: agent_address,
        logo,
        content_type,
        spaced_rune,
        premine: total_supply,
        divisibility: 3,
        symbol,
        turbo: true,
        fee_payer,
        fee_per_vbytes,
        fee_payer_account: account,
    })
    .await
    {
        Err(required_balance) => {
            write_agents(|agents| {
                agents.delete_agent(id);
            });
            ic_cdk::println!("required balance: {}", required_balance);
            ic_cdk::trap("not enough balance")
        }
        Ok((handler, (commit, reveal))) => {
            write_agents(|agents| {
                let mut agent = agents.mapping.get(&id).expect("should exist");
                agent.txns.0.replace(commit);
                agent.txns.1.replace(reveal);
                agents.mapping.insert(id, agent);
            });
            handler
        }
    };
    handler.submit().await;
    id
}

#[derive(CandidType, Deserialize)]
pub enum AgentBy {
    Id(u128),
    Name(String),
}

#[derive(CandidType, Deserialize)]
pub struct BuyArgs {
    pub id: AgentBy,
    pub buy_exact_in: u64,
    pub amount_out_min: u128,
}

#[update]
pub async fn buy(
    BuyArgs {
        id,
        buy_exact_in,
        amount_out_min,
    }: BuyArgs,
) -> u128 {
    let caller = ic_cdk::caller();
    let account = utils::get_account_for(&caller);
    let bitcoin_address = bitcoin::account_to_p2pkh_address(&account);
    indexer::fetch_utxos_and_update(
        &bitcoin_address,
        indexer::TargetType::Bitcoin { target: u64::MAX },
    )
    .await;
    let bitcoin_balance = read_ledger_entries(|entries| {
        let entry = entries.get(&caller).unwrap_or_default();
        let balance = read_utxo_manager(|manager| manager.get_bitcoin_balance(&bitcoin_address));
        balance - entry.restricted_bitcoin_balance
    });
    if buy_exact_in > bitcoin_balance {
        ic_cdk::trap("not enough balance")
    }
    let (id, amount) = write_agents(|agents| {
        let id = agents.find_agent_id(id).expect("invalid agent id");
        let mut agent = agents.mapping.get(&id).unwrap();
        let amount = agent
            .buy_exact_in(buy_exact_in as u128, amount_out_min)
            .unwrap();
        agent.balances.insert(caller.to_text());
        agents.mapping.insert(id, agent);
        (id, amount)
    });
    write_ledger_entries(|entries| {
        let mut entry = entries.get(&caller).unwrap_or_default();
        entry.record_agent_owned_balance(id, buy_exact_in);
        entry.record_rune_balance(id, amount);
        entries.insert(caller, entry);
    });
    amount
}

#[derive(CandidType, Deserialize)]
pub struct SellArgs {
    pub id: AgentBy,
    pub token_amount: u128,
    pub amount_collateral_min: u64,
}

#[update]
pub fn sell(
    SellArgs {
        id,
        token_amount,
        amount_collateral_min,
    }: SellArgs,
) -> u128 {
    let caller = ic_cdk::caller();
    let id = read_agents(|agents| agents.find_agent_id(id)).expect("invalid agent");
    let rune_balance = read_ledger_entries(|entries| {
        entries
            .get(&caller)
            .unwrap_or_default()
            .ledger_entries
            .get(&id)
            .copied()
            .unwrap_or_default()
            .1
    });
    if token_amount > rune_balance {
        ic_cdk::trap("not enough balance")
    }
    let bitcoin = write_agents(|agents| {
        let mut agent = agents.mapping.get(&id).unwrap();
        let bitcoin = agent
            .sell_exact_in(token_amount, amount_collateral_min as u128)
            .unwrap();
        agents.mapping.insert(id, agent);
        bitcoin
    });
    write_ledger_entries(|entries| {
        let mut entry = entries.get(&caller).unwrap_or_default();
        entry.deduct_rune_balance(id, token_amount);
        entry.deduct_agent_owned_balance(id, bitcoin as u64);
        entries.insert(caller, entry);
    });
    bitcoin
}

#[derive(CandidType, Deserialize)]
pub struct LuckyDraw {
    pub id: AgentBy,
    pub message: String,
}

#[update]
pub fn lucky_draw(LuckyDraw { id, message }: LuckyDraw) -> String {
    let caller = ic_cdk::caller();
    write_agents(|agents| {
        let id = agents.find_agent_id(id).expect("agent doesn't exist");
        let mut agent = agents.mapping.get(&id).unwrap();
        if agent.current_winner.is_some() {
            return String::from("Contest not started");
        }
        if Some(message) == agent.secret {
            agent.current_winner.replace(caller);
            String::from("Congratulation")
        } else {
            String::from("Better luck next time")
        }
    })
}

#[update]
pub fn create_chat_session(agent: AgentBy) -> u128 {
    let caller = ic_cdk::caller();
    let agent_id = read_agents(|agents| agents.find_agent_id(agent)).expect("agent doesn't exist");
    write_chat_session(|session| session.start_new_session(agent_id, caller))
}

#[derive(CandidType, Deserialize)]
pub struct ChatArgs {
    pub agent: AgentBy,
    pub session_id: u128,
    pub message: String,
}

#[update]
pub async fn chat(
    ChatArgs {
        agent,
        session_id,
        message,
    }: ChatArgs,
) -> String {
    let caller = ic_cdk::caller();
    let agent_id = read_agents(|agents| agents.find_agent_id(agent)).expect("agent doesn't exist");
    let account = utils::get_account_for(&caller);
    let user_bitcoin_address = bitcoin::account_to_p2pkh_address(&account);
    let (bitcoin, rune) = read_ledger_entries(|entries| {
        let entry = entries.get(&caller).unwrap_or_default();
        let balance =
            read_utxo_manager(|manager| manager.get_bitcoin_balance(&user_bitcoin_address));
        let bitcoin = balance - entry.restricted_bitcoin_balance;
        let rune = entry
            .ledger_entries
            .get(&agent_id)
            .copied()
            .unwrap_or_default()
            .1;
        (bitcoin, rune)
    });
    llm::Llm::chat(
        session_id,
        agent_id,
        user_bitcoin_address,
        bitcoin,
        rune,
        message,
    )
    .await
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

fn get_agent_id(url: &str) -> Option<u128> {
    let url_split_by_path: Vec<&str> = url.split('/').collect();
    let last_elem = url_split_by_path.last()?; // Safe access
    let first_elem = last_elem.split('?').next()?; // Extract ID before query params
    first_elem.trim().parse::<u128>().ok() // Safe parsing
}

#[query]
pub fn http_request(req: HttpRequest) -> HttpResponse {
    let agent_id = match get_agent_id(&req.url) {
        Some(id) => id,
        None => {
            return HttpResponse {
                body: b"Invalid Agent ID".to_vec(),
                status_code: 400,
                headers: vec![],
                streaming_strategy: None,
            };
        }
    };

    let not_found = HttpResponse {
        body: b"Asset not Found".to_vec(),
        status_code: 404,
        headers: vec![],
        streaming_strategy: None,
    };

    read_agents(|agents| match agents.mapping.get(&agent_id) {
        None => not_found,
        Some(agent) => match &agent.logo {
            None => not_found,
            Some(image) => {
                let image_bytes = if image.starts_with("data:image/") {
                    // If it's Base64 encoded, decode it
                    let base64_data = image.split(',').nth(1); // Extract base64 part
                    match base64_data {
                        Some(data) => match base64::decode(data) {
                            Ok(decoded) => decoded,
                            Err(_) => return not_found, // Decoding failed
                        },
                        None => return not_found, // Invalid Base64 URI
                    }
                } else {
                    image.as_bytes().to_vec() // Assume it's already raw bytes
                };

                HttpResponse {
                    body: image_bytes,
                    status_code: 200,
                    headers: vec![HeaderField(
                        "Content-Type".to_string(),
                        "image/png".to_string(), // Adjust based on actual format
                    )],
                    streaming_strategy: None,
                }
            }
        },
    })
}

ic_cdk::export_candid!();
