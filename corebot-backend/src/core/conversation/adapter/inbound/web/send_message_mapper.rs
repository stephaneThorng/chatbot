use super::send_message_dto::{SendMessageRequest, SendMessageResponse};
use crate::core::conversation::application::dto::conversation_command::{
    HandleConversationCommand, HandleConversationResult,
};

impl From<SendMessageRequest> for HandleConversationCommand {
    fn from(req: SendMessageRequest) -> Self {
        HandleConversationCommand {
            message: req.message,
            session_id: req.session_id,
        }
    }
}

impl From<HandleConversationResult> for SendMessageResponse {
    fn from(result: HandleConversationResult) -> Self {
        SendMessageResponse {
            session_id: result.session_id,
            reply: result.reply,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_maps_to_command_with_session_id() {
        let req = SendMessageRequest {
            message: "hello".to_string(),
            session_id: Some("sess-1".to_string()),
        };
        let cmd: HandleConversationCommand = req.into();
        assert_eq!(cmd.message, "hello");
        assert_eq!(cmd.session_id, Some("sess-1".to_string()));
    }

    #[test]
    fn request_maps_to_command_without_session_id() {
        let req = SendMessageRequest {
            message: "hi".to_string(),
            session_id: None,
        };
        let cmd: HandleConversationCommand = req.into();
        assert_eq!(cmd.session_id, None);
    }

    #[test]
    fn result_maps_to_response() {
        let result = HandleConversationResult {
            session_id: "sess-42".to_string(),
            reply: vec!["Hello!".to_string()],
        };
        let response: SendMessageResponse = result.into();
        assert_eq!(response.session_id, "sess-42");
        assert_eq!(response.reply, vec!["Hello!".to_string()]);
    }
}
