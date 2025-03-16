use super::{CanisterMemory, CanisterMemoryIds, read_memory_manager};
use ic_stable_structures::StableBTreeMap;

pub type Commission = StableBTreeMap<u128, u64, CanisterMemory>;

pub fn init_commission() -> Commission {
    read_memory_manager(|manager| {
        let memory = manager.get(CanisterMemoryIds::Commission.into());
        Commission::init(memory)
    })
}
