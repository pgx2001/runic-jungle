use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

const LLM_CANISTER: &str = "w36hm-eqaaa-aaaal-qr76a-cai";
const TOOLS: &str = r#"
"#;

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(CandidType, Deserialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

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
