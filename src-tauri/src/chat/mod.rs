pub mod commands;
pub mod llm_client;

// Re-exports (available for external use if needed)
pub use llm_client::{LLMChatClient, ChatRequest, ChatMessage};

