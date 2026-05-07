use uuid::Uuid;

use super::conversation_command::{HandleConversationCommand, HandleConversationResult};
use super::port::input::conversation_trait::HandleConversation;

pub struct HandleConversationUseCase;

impl HandleConversation for HandleConversationUseCase {
    fn handle_message(&self, command: HandleConversationCommand) -> HandleConversationResult {
        let session_id = command
            .session_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        HandleConversationResult {
            session_id,
            reply: "Not yet implemented".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_command(session_id: Option<&str>) -> HandleConversationCommand {
        HandleConversationCommand {
            message: "hello".to_string(),
            session_id: session_id.map(str::to_string),
        }
    }

    #[test]
    fn handle_message_reuses_provided_session_id() {
        let result = HandleConversationUseCase.handle_message(make_command(Some("existing-123")));
        assert_eq!(result.session_id, "existing-123");
    }

    #[test]
    fn handle_message_generates_session_id_when_none() {
        let result = HandleConversationUseCase.handle_message(make_command(None));
        assert!(!result.session_id.is_empty());
    }

    #[test]
    fn handle_message_generates_unique_session_ids() {
        let a = HandleConversationUseCase.handle_message(make_command(None));
        let b = HandleConversationUseCase.handle_message(make_command(None));
        assert_ne!(a.session_id, b.session_id);
    }

    #[test]
    fn handle_message_returns_stub_reply() {
        let result = HandleConversationUseCase.handle_message(make_command(None));
        assert_eq!(result.reply, "Not yet implemented");
    }
}

