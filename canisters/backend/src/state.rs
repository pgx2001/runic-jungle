use std::cell::RefCell;

use config::{StableConfig, initialize_config};
use ic_stable_structures::{
    DefaultMemoryImpl,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
};

mod agent;
mod chat_session;
mod config;

type CanisterMemory = VirtualMemory<DefaultMemoryImpl>;

pub enum CanisterMemoryIds {
    Config = 0,
    Agent = 1,
    AssociatedAgentSet = 2,
    ChatSession = 3,
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
}

// helper functions

pub fn read_memory_manager<F, R>(f: F) -> R
where
    F: FnOnce(&MemoryManager<DefaultMemoryImpl>) -> R,
{
    MEMORY_MANAGER.with_borrow(|manager| f(manager))
}
