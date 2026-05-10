use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::domain_gateway_trait::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};

pub struct OpeningHoursIntentHandler {
    domain_gateway: Arc<dyn DomainGatewayPort>,
}

impl OpeningHoursIntentHandler {
    pub fn new(domain_gateway: Arc<dyn DomainGatewayPort>) -> Self {
        Self { domain_gateway }
    }
}

impl IntentHandler for OpeningHoursIntentHandler {
    fn intent(&self) -> IntentId {
        IntentId::AskOpeningHours
    }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy {
            id: self.intent(),
            kind: IntentKind::Informational,
            nlu_task: None,
            workflow_slots: vec![],
            confirmation_prompt: None,
            completion_response: None,
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let _ = (
            input.conversation,
            input.analysis_intent,
            input.text,
            input.conversation.lang.as_str(),
            input.analysis_entities,
        );
        StateHandlerResult {
            updated_conversation: input.conversation.clone(),
            reply: self.domain_gateway.get_opening_hours(),
            handled_intent: self.intent(),
        }
    }
}
