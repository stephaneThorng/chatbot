use super::send_message_dto::{SendMessageRequest, SendMessageResponse};
use crate::core::conversation::application::{HandleConversationCommand, HandleConversationResult};

impl From<SendMessageRequest> for HandleConversationCommand {
    fn from(req: SendMessageRequest) -> Self {
        HandleConversationCommand {
            message: req.message,
            session_id: req.session_id,
            lang: req.lang,
            task: req.task,
        }
    }
}

impl From<HandleConversationResult> for SendMessageResponse {
    fn from(result: HandleConversationResult) -> Self {
        SendMessageResponse {
            session_id: result.session_id,
            reply: result.reply,
            detected_intent: result.detected_intent,
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
            lang: Some("en".to_string()),
            task: None,
        };
        let cmd: HandleConversationCommand = req.into();
        assert_eq!(cmd.message, "hello");
        assert_eq!(cmd.session_id, Some("sess-1".to_string()));
        assert_eq!(cmd.lang, Some("en".to_string()));
    }

    #[test]
    fn request_maps_to_command_without_session_id() {
        let req = SendMessageRequest {
            message: "hi".to_string(),
            session_id: None,
            lang: None,
            task: None,
        };
        let cmd: HandleConversationCommand = req.into();
        assert_eq!(cmd.session_id, None);
        assert_eq!(cmd.lang, None);
    }

    #[test]
    fn result_maps_to_response() {
        let result = HandleConversationResult {
            session_id: "sess-42".to_string(),
            reply: "Hello!".to_string(),
            detected_intent: "greeting".to_string(),
        };
        let response: SendMessageResponse = result.into();
        assert_eq!(response.session_id, "sess-42");
        assert_eq!(response.reply, "Hello!");
        assert_eq!(response.detected_intent, "greeting");
    }
}
