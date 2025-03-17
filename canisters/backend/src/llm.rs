use crate::{
    bitcoin::account_to_p2pkh_address,
    state::{read_agents, read_chat_session, write_chat_session},
};
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
    pub async fn generate_secret_word(agent_name: &str, agent_description: &str) -> String {
        let system = format!(
            r#"You're an AI agent. Your name is {agent_name}. Here is some more description about you: {agent_description}. Your task is to generate some word for a lucky draw contest. If any user is able to guess the word, they win the prize.
    RULES for generating the word:
    1: number of words can range from 1 to 5. for e.g ("{agent_name}", "{agent_name} is awesome")
    2: generated word should be related to the agent's name, description.

    NOTE: return the secret word only.
"#
        );
        let llm_canister = Principal::from_text(LLM_CANISTER).unwrap();
        let contents = vec![
            ChatMessage {
                role: Role::System,
                content: system,
            },
            ChatMessage {
                role: Role::User,
                content: String::from("generate a word"),
            },
        ];
        let arg = LlmRequest {
            model: ic_llm::Model::Llama3_1_8B.to_string(),
            messages: contents,
        };
        ic_cdk::call::<(LlmRequest,), (String,)>(llm_canister, "v0_chat", (arg,))
            .await
            .unwrap()
            .0
    }

    pub async fn chat(
        session_id: u128,
        agent_id: u128,
        user_bitcoin_address: String,
        message: String,
    ) -> String {
        let (agent_name, agent_description) = read_agents(|agents| {
            let agent = agents.mapping.get(&agent_id).unwrap();
            (agent.name, agent.description)
        });
        let system = format!(
            r#"You're a helpful AI agent. Your name is {agent_name}. Here is some more description about you: {agent_description}. The bitcoin wallet address of user is: {user_bitcoin_address}."#
        );
        let llm_canister = Principal::from_text(LLM_CANISTER).unwrap();
        let messages = read_chat_session(|sessions| {
            let session = sessions.session.get(&session_id).unwrap();
            let mut messages = vec![ChatMessage {
                role: Role::System,
                content: system,
            }];
            messages.extend_from_slice(&session.to_contents());
            messages
        });
        let arg = LlmRequest {
            model: ic_llm::Model::Llama3_1_8B.to_string(),
            messages,
        };
        let response = ic_cdk::call::<(LlmRequest,), (String,)>(llm_canister, "v0_chat", (arg,))
            .await
            .unwrap()
            .0;
        ic_cdk::println!("{:?}", response);
        write_chat_session(|sessions| {
            let mut session = sessions.session.get(&session_id).unwrap();
            session.record_content(vec![
                ChatMessage {
                    role: Role::User,
                    content: message,
                },
                ChatMessage {
                    role: Role::Assistant,
                    content: response.clone(),
                },
            ]);
            session.last_interacted = ic_cdk::api::time();
            sessions.session.insert(session_id, session);
        });
        response
    }
}
