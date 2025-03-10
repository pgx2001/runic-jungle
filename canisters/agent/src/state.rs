use config::{Config, StableConfig, init_stable_config};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory}, DefaultMemoryImpl
};
use std::cell::RefCell;

mod prize_pool;
mod config;
mod market_maker;

type Memory = VirtualMemory<DefaultMemoryImpl>;

pub enum MemoryIds{
    Config = 0,
    PrizePool = 1,
    MarketMaker = 2,
}

impl From<MemoryIds> for MemoryId{
    fn from(value: MemoryIds) -> Self {
        let id = value as u8;
        MemoryId::new(id)
    }
}

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    pub static CONFIG: RefCell<StableConfig> = RefCell::new(init_stable_config());
    pub static PRIZE_POOL: RefCell<prize_pool::PrizePool> = RefCell::default();
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
    F: FnOnce(&StableConfig) -> R,
{
    CONFIG.with_borrow_mut(|config| f(config))
}
