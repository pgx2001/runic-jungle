use candid::Principal;

pub struct Llm {
    pub principal: Principal,
}

impl Llm {
    pub fn new() -> Self {
        todo!()
    }

    pub fn chat(&mut self) {}

    pub fn generate_action(&mut self) {}
}
