pub mod commands;
pub mod llm_client;

pub use commands::{chat_with_ai, check_llm_health, get_llm_stats};
pub use llm_client::{LLMChatClient, ChatRequest, ChatResponse, ChatMessage};

