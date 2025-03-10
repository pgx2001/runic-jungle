use candid::{CandidType, Principal};
use ic_llm::Model;
use serde::{Deserialize, Serialize};

use crate::read_config;

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

pub struct ICLLM {
    pub system_prompt: String,
    principal: Principal,
    past_messages: Vec<ChatMessage>,
}

impl ICLLM {
    pub fn new(for_chat: bool) -> Self {
        let principal = Principal::from_text(LLM_CANISTER).unwrap();
        let system_prompt = if for_chat {
            read_config(|config| config.get_system_prompt())
        } else {
            format!(
                "{}{TOOLS}",
                read_config(|config| config.get_system_prompt())
            )
        };
        Self {
            principal,
            system_prompt,
            past_messages: vec![],
        }
    }

    pub async fn chat(&mut self, content: String) -> String {
        let mut messages = vec![ChatMessage {
            content: self.system_prompt.clone(),
            role: Role::System,
        }];
        messages.extend_from_slice(&self.past_messages);
        let content = ChatMessage {
            role: Role::User,
            content,
        };
        messages.push(content.clone());
        self.past_messages.push(content);

        let response = ic_cdk::api::call::call::<(LlmRequest,), (String,)>(
            self.principal,
            "v0_chat",
            (LlmRequest {
                model: Model::Llama3_1_8B.to_string(),
                messages,
            },),
        )
        .await
        .expect("failed to call llm canister")
        .0;
        self.past_messages.push(ChatMessage {
            content: response.clone(),
            role: Role::Assistant,
        });
        response
    }
}
