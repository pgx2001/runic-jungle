use std::collections::HashMap;

pub struct Chat {
    pub session_id: u128,
    pub past: HashMap<u32, (bool, String)>,
}

impl Chat {
    pub fn to_contents(&self) -> Vec<()> {
        todo!()
    }
}
