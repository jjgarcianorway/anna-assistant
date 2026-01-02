//! Specialist Conversation Display - Phase 78
//!
//! Tracks and displays conversations between Anna and specialists.
//! VISION.md shows the "fly on the wall" experience of internal communications.

pub mod conversation;
pub mod formatting;
pub mod history;
pub mod types;
pub mod utils;

// Re-export main types
pub use conversation::Conversation;
pub use formatting::{
    format_conversation, format_conversation_history, format_conversation_history_compact,
    format_conversation_history_oneline,
};
pub use history::ConversationHistory;
pub use types::{ConversationMessage, MessageType, Speaker};
pub use utils::{conversation_fun_fact, is_conversation_query};
