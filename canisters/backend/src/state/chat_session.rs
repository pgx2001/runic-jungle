use super::{CanisterMemory, CanisterMemoryIds, read_memory_manager};
use crate::llm::{ChatMessage, Role};
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{StableBTreeMap, Storable, storable::Bound};
use std::collections::HashMap;

#[derive(CandidType, Deserialize)]
pub struct Session {
    pub session_id: u128,
    pub agent_id: u128,
    pub last_interacted: u64, // chat's will be deleted after 30 mins of no action
    pub user: Principal,
    pub past: HashMap<u32, (bool, String)>, // true if the message is by agent
}

impl Storable for Session {
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("should decode")
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).expect("should encode"))
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Session {
    pub fn new(id: u128, agent_id: u128, user: Principal) -> Self {
        Self {
            session_id: id,
            agent_id,
            user,
            last_interacted: ic_cdk::api::time(),
            past: HashMap::new(),
        }
    }

    pub fn to_contents(&self) -> Vec<ChatMessage> {
        let count = self.past.len();
        let mut messages = Vec::with_capacity(count);

        for i in 0..count {
            let message = self.past.get(&(i as u32)).expect("should exist");
            messages.push(ChatMessage {
                content: message.1.clone(),
                role: if message.0 {
                    Role::Assistant
                } else {
                    Role::User
                },
            });
        }
        messages
    }

    pub fn record_content(&mut self, contents: Vec<ChatMessage>) {
        for content in contents {
            let id = self.past.len() as u32;
            let by_agent = match content.role {
                Role::Assistant => true,
                _ => false,
            };
            self.past.insert(id, (by_agent, content.content));
        }
    }
}

pub struct ChatSession {
    pub count: u128,
    pub session: StableBTreeMap<u128, Session, CanisterMemory>,
}

impl Default for ChatSession {
    fn default() -> Self {
        read_memory_manager(|manager| Self {
            count: 0,
            session: StableBTreeMap::init(manager.get(CanisterMemoryIds::ChatSession.into())),
        })
    }
}

impl ChatSession {
    fn get_session_id(&mut self) -> u128 {
        let id = self.count;
        self.count += 1;
        id
    }

    pub fn start_new_session(&mut self, agent: u128, user: Principal) -> u128 {
        let id = self.get_session_id();
        let session = Session::new(id, agent, user);
        self.session.insert(id, session);
        id
    }
}
