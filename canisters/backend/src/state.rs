use std::cell::RefCell;

use config::{StableConfig, initialize_config};
use ic_stable_structures::{
    DefaultMemoryImpl,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
};

mod agent;
mod chat_session;
mod commission;
mod config;
mod ledger_entries;
pub mod queue;
pub mod utxo_manager;

use agent::AgentState;
use chat_session::ChatSession;
use commission::{Commission, init_commission};
use config::Config;
use ledger_entries::{LedgerEntries, init_ledger_entries};
use queue::ScheduledState;
use utxo_manager::UtxoManager;

type CanisterMemory = VirtualMemory<DefaultMemoryImpl>;

pub enum CanisterMemoryIds {
    Config = 0,
    Agent = 1,
    AssociatedAgentSet = 2,
    ChatSession = 3,
    LedgerEntries = 4,
    Bitcoin = 5,
    Runic = 6,
    Queue = 7,
    Commission = 8,
}

impl From<CanisterMemoryIds> for MemoryId {
    fn from(value: CanisterMemoryIds) -> Self {
        let id = value as u8;
        MemoryId::new(id)
    }
}

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    pub static CONFIG: RefCell<StableConfig> = RefCell::new(initialize_config());
    pub static AGENTS: RefCell<AgentState> = RefCell::default();
    pub static CHAT_SESSION: RefCell<ChatSession> = RefCell::default();
    pub static LEDGER_ENTRIES: RefCell<LedgerEntries> = RefCell::new(init_ledger_entries());
    pub static UTXO_MANAGER: RefCell<UtxoManager> = RefCell::default();
    pub static SCHEDULED_STATE: RefCell<ScheduledState> = RefCell::default();
    pub static COMMISSION: RefCell<Commission> = RefCell::new(init_commission());
}

// helper functions

pub fn read_memory_manager<F, R>(f: F) -> R
where
    F: FnOnce(&MemoryManager<DefaultMemoryImpl>) -> R,
{
    MEMORY_MANAGER.with_borrow(|manager| f(manager))
}

pub fn read_config<F, R>(f: F) -> R
where
    F: FnOnce(&Config) -> R,
{
    CONFIG.with_borrow(|config| f(config.get()))
}

pub fn write_config<F, R>(f: F) -> R
where
    F: FnOnce(&mut StableConfig) -> R,
{
    CONFIG.with_borrow_mut(|config| f(config))
}

pub fn read_agents<F, R>(f: F) -> R
where
    F: FnOnce(&AgentState) -> R,
{
    AGENTS.with_borrow(|state| f(state))
}

pub fn write_agents<F, R>(f: F) -> R
where
    F: FnOnce(&mut AgentState) -> R,
{
    AGENTS.with_borrow_mut(|state| f(state))
}

pub fn read_chat_session<F, R>(f: F) -> R
where
    F: FnOnce(&ChatSession) -> R,
{
    CHAT_SESSION.with_borrow(|session| f(session))
}

pub fn write_chat_session<F, R>(f: F) -> R
where
    F: FnOnce(&mut ChatSession) -> R,
{
    CHAT_SESSION.with_borrow_mut(|session| f(session))
}

pub fn read_ledger_entries<F, R>(f: F) -> R
where
    F: FnOnce(&LedgerEntries) -> R,
{
    LEDGER_ENTRIES.with_borrow(|entries| f(entries))
}

pub fn write_ledger_entries<F, R>(f: F) -> R
where
    F: FnOnce(&mut LedgerEntries) -> R,
{
    LEDGER_ENTRIES.with_borrow_mut(|entries| f(entries))
}

pub fn read_utxo_manager<F, R>(f: F) -> R
where
    F: FnOnce(&UtxoManager) -> R,
{
    UTXO_MANAGER.with_borrow(|manager| f(manager))
}

pub fn write_utxo_manager<F, R>(f: F) -> R
where
    F: FnOnce(&mut UtxoManager) -> R,
{
    UTXO_MANAGER.with_borrow_mut(|manager| f(manager))
}

pub fn read_scheduled_state<F, R>(f: F) -> R
where
    F: FnOnce(&ScheduledState) -> R,
{
    SCHEDULED_STATE.with_borrow(|state| f(state))
}

pub fn write_scheduled_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut ScheduledState) -> R,
{
    SCHEDULED_STATE.with_borrow_mut(|state| f(state))
}

pub fn read_commission_state<F, R>(f: F) -> R
where
    F: FnOnce(&Commission) -> R,
{
    COMMISSION.with_borrow(|commission| f(commission))
}

pub fn write_commission_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut Commission) -> R,
{
    COMMISSION.with_borrow_mut(|commission| f(commission))
}
