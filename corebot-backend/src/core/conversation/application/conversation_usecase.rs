use std::sync::Arc;

use uuid::Uuid;

use super::conversation_command::{HandleConversationCommand, HandleConversationResult};
use super::port::inbound::conversation_trait::HandleConversationPort;
use super::port::outbound::domain_gateway_trait::DomainGatewayPort;
use super::port::outbound::nlp_analyzer_trait::NlpEngineGatewayPort;
pub struct HandleConversationUseCase {
    domain_gateway: Arc<dyn DomainGatewayPort>,
    nlu_engine_gateway: Arc<dyn NlpEngineGatewayPort>,
}

impl HandleConversationUseCase {
    pub fn new(
        domain_gateway: Arc<dyn DomainGatewayPort>,
        nlu_engine_gateway: Arc<dyn NlpEngineGatewayPort>,
    ) -> Self {
        Self {
            domain_gateway,
            nlu_engine_gateway,
        }
    }
}

impl HandleConversationPort for HandleConversationUseCase {
    fn handle_message(&self, command: HandleConversationCommand) -> HandleConversationResult {
        let session_id = command
            .session_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let analysis = self.nlu_engine_gateway.analyze(
            &command.message,
            command.lang.as_deref().unwrap_or("en"),
            "restaurant",
            command.task,
        );

        let reply = match analysis.intent.name.as_str() {
            "opening_hours" => self.domain_gateway.get_opening_hours(),
            "greeting" => "Hello! How can I help with the restaurant today?".to_string(),
            "thanks" => "You're welcome.".to_string(),
            "farewell" => "Goodbye!".to_string(),
            _ => format!("Detected intent: {}", analysis.intent.name),
        };

        HandleConversationResult {
            session_id,
            reply,
            detected_intent: analysis.intent.name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nlu_engine::domain::analysis::{NluAnalysis, NluIntent};

    struct StubDomainGateway;

    impl DomainGatewayPort for StubDomainGateway {
        fn get_opening_hours(&self) -> String {
            "Mon-Sun 9am-10pm".to_string()
        }
    }

    struct StubNlpAnalyzer {
        intent_name: &'static str,
    }

    impl NlpEngineGatewayPort for StubNlpAnalyzer {
        fn analyze(
            &self,
            text: &str,
            lang: &str,
            domain: &str,
            task: Option<String>,
        ) -> NluAnalysis {
            let _ = (lang, domain, task);
            NluAnalysis {
                tagged_text: text.to_string(),
                intent: NluIntent {
                    name: self.intent_name.to_string(),
                    confidence: 1.0,
                },
                intents: vec![],
                entities: vec![],
                ner_labels: vec![],
            }
        }
    }

    fn make_use_case(intent_name: &'static str) -> HandleConversationUseCase {
        HandleConversationUseCase::new(
            Arc::new(StubDomainGateway),
            Arc::new(StubNlpAnalyzer { intent_name }),
        )
    }

    fn make_command(session_id: Option<&str>) -> HandleConversationCommand {
        HandleConversationCommand {
            message: "hello".to_string(),
            session_id: session_id.map(str::to_string),
            lang: Some("en".to_string()),
            task: None,
        }
    }

    #[test]
    fn handle_message_reuses_provided_session_id() {
        let result = make_use_case("greeting").handle_message(make_command(Some("existing-123")));
        assert_eq!(result.session_id, "existing-123");
    }

    #[test]
    fn handle_message_generates_session_id_when_none() {
        let result = make_use_case("greeting").handle_message(make_command(None));
        assert!(!result.session_id.is_empty());
    }

    #[test]
    fn handle_message_generates_unique_session_ids() {
        let uc = make_use_case("greeting");
        let a = uc.handle_message(make_command(None));
        let b = uc.handle_message(make_command(None));
        assert_ne!(a.session_id, b.session_id);
    }

    #[test]
    fn handle_message_delegates_opening_hours_reply_to_domain_gateway() {
        let result = make_use_case("opening_hours").handle_message(make_command(None));
        assert_eq!(result.reply, "Mon-Sun 9am-10pm");
    }

    #[test]
    fn handle_message_returns_detected_intent() {
        let result = make_use_case("thanks").handle_message(make_command(None));
        assert_eq!(result.detected_intent, "thanks");
    }
}
