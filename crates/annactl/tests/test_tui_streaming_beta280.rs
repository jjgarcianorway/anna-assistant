//! Beta.280: TUI Streaming Tests
//!
//! Tests for message streaming lifecycle to ensure:
//! - One message per reply
//! - No duplicate rendering
//! - Proper Draft -> Final state transitions

use annactl::tui_state::{AnnaTuiState, ChatItem};

#[test]
fn test_streaming_lifecycle_single_message() {
    let mut state = AnnaTuiState::default();

    // Start streaming reply
    state.start_streaming_reply();

    // Should have exactly one message in Draft state
    assert_eq!(state.conversation.len(), 1);
    match &state.conversation[0] {
        ChatItem::Anna { text, is_streaming } => {
            assert_eq!(text, "");
            assert!(is_streaming, "Message should be in streaming state");
        }
        _ => panic!("Expected Anna message"),
    }

    // Append chunks
    state.append_to_last_anna_reply("Hello ".to_string());
    state.append_to_last_anna_reply("world!".to_string());

    // Still exactly one message
    assert_eq!(state.conversation.len(), 1);
    match &state.conversation[0] {
        ChatItem::Anna { text, is_streaming } => {
            assert_eq!(text, "Hello world!");
            assert!(is_streaming, "Message should still be streaming");
        }
        _ => panic!("Expected Anna message"),
    }

    // Complete streaming
    state.complete_streaming_reply();

    // Still exactly one message, now finalized
    assert_eq!(state.conversation.len(), 1);
    match &state.conversation[0] {
        ChatItem::Anna { text, is_streaming } => {
            assert_eq!(text, "Hello world!");
            assert!(!is_streaming, "Message should be finalized");
        }
        _ => panic!("Expected Anna message"),
    }
}

#[test]
fn test_two_consecutive_streaming_replies() {
    let mut state = AnnaTuiState::default();

    // First reply
    state.start_streaming_reply();
    state.append_to_last_anna_reply("First ".to_string());
    state.append_to_last_anna_reply("reply".to_string());
    state.complete_streaming_reply();

    // Second reply
    state.start_streaming_reply();
    state.append_to_last_anna_reply("Second ".to_string());
    state.append_to_last_anna_reply("reply".to_string());
    state.complete_streaming_reply();

    // Should have exactly two messages
    assert_eq!(state.conversation.len(), 2);

    match &state.conversation[0] {
        ChatItem::Anna { text, is_streaming } => {
            assert_eq!(text, "First reply");
            assert!(!is_streaming);
        }
        _ => panic!("Expected Anna message"),
    }

    match &state.conversation[1] {
        ChatItem::Anna { text, is_streaming } => {
            assert_eq!(text, "Second reply");
            assert!(!is_streaming);
        }
        _ => panic!("Expected Anna message"),
    }
}

#[test]
fn test_append_creates_message_if_none_exists() {
    let mut state = AnnaTuiState::default();

    // Append without calling start_streaming_reply first
    state.append_to_last_anna_reply("Emergency message".to_string());

    // Should create a new streaming message
    assert_eq!(state.conversation.len(), 1);
    match &state.conversation[0] {
        ChatItem::Anna { text, is_streaming } => {
            assert_eq!(text, "Emergency message");
            assert!(is_streaming, "Auto-created message should be in streaming state");
        }
        _ => panic!("Expected Anna message"),
    }
}

#[test]
fn test_non_streaming_reply_still_works() {
    let mut state = AnnaTuiState::default();

    // Add non-streaming reply (full message at once)
    state.add_anna_reply("Complete message".to_string());

    assert_eq!(state.conversation.len(), 1);
    match &state.conversation[0] {
        ChatItem::Anna { text, is_streaming } => {
            assert_eq!(text, "Complete message");
            assert!(!is_streaming, "Non-streaming messages should be marked as complete");
        }
        _ => panic!("Expected Anna message"),
    }
}

#[test]
fn test_mixed_user_and_anna_messages() {
    let mut state = AnnaTuiState::default();

    // User message
    state.add_user_message("Hello Anna".to_string());

    // Streaming Anna reply
    state.start_streaming_reply();
    state.append_to_last_anna_reply("Hi there!".to_string());
    state.complete_streaming_reply();

    // Another user message
    state.add_user_message("How are you?".to_string());

    // Non-streaming Anna reply
    state.add_anna_reply("I'm doing well!".to_string());

    // Should have exactly 4 messages in conversation
    assert_eq!(state.conversation.len(), 4);

    // Verify no duplicates or extra messages
    match &state.conversation[0] {
        ChatItem::User(_) => {},
        _ => panic!("Expected User message at index 0"),
    }

    match &state.conversation[1] {
        ChatItem::Anna { text, .. } => assert_eq!(text, "Hi there!"),
        _ => panic!("Expected Anna message at index 1"),
    }

    match &state.conversation[2] {
        ChatItem::User(_) => {},
        _ => panic!("Expected User message at index 2"),
    }

    match &state.conversation[3] {
        ChatItem::Anna { text, .. } => assert_eq!(text, "I'm doing well!"),
        _ => panic!("Expected Anna message at index 3"),
    }
}
