use std::sync::Arc;

use uuid::Uuid;

use super::conversation_command::{HandleConversationCommand, HandleConversationResult};
use super::port::input::conversation_trait::HandleConversation;
use super::port::output::domain_gateway_trait::DomainGateway;

pub struct HandleConversationUseCase {
    domain_gateway: Arc<dyn DomainGateway>,
}

impl HandleConversationUseCase {
    pub fn new(domain_gateway: Arc<dyn DomainGateway>) -> Self {
        Self { domain_gateway }
    }
}

impl HandleConversation for HandleConversationUseCase {
    fn handle_message(&self, command: HandleConversationCommand) -> HandleConversationResult {
        let session_id = command
            .session_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Stub intent routing — replace with NLP dispatch when available.
        let reply = self.domain_gateway.get_opening_hours();

        HandleConversationResult { session_id, reply }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubDomainGateway;

    impl DomainGateway for StubDomainGateway {
        fn get_opening_hours(&self) -> String {
            "Mon-Sun 9am-10pm".to_string()
        }
    }

    fn make_use_case() -> HandleConversationUseCase {
        HandleConversationUseCase::new(Arc::new(StubDomainGateway))
    }

    fn make_command(session_id: Option<&str>) -> HandleConversationCommand {
        HandleConversationCommand {
            message: "hello".to_string(),
            session_id: session_id.map(str::to_string),
        }
    }

    #[test]
    fn handle_message_reuses_provided_session_id() {
        let result = make_use_case().handle_message(make_command(Some("existing-123")));
        assert_eq!(result.session_id, "existing-123");
    }

    #[test]
    fn handle_message_generates_session_id_when_none() {
        let result = make_use_case().handle_message(make_command(None));
        assert!(!result.session_id.is_empty());
    }

    #[test]
    fn handle_message_generates_unique_session_ids() {
        let uc = make_use_case();
        let a = uc.handle_message(make_command(None));
        let b = uc.handle_message(make_command(None));
        assert_ne!(a.session_id, b.session_id);
    }

    #[test]
    fn handle_message_delegates_reply_to_domain_gateway() {
        let result = make_use_case().handle_message(make_command(None));
        assert_eq!(result.reply, "Mon-Sun 9am-10pm");
    }
}

