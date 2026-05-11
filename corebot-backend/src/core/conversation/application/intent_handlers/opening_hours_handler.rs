use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};

pub struct OpeningHoursIntentHandler<D: DomainGatewayPort> {
    domain_gateway: D,
}

impl<D: DomainGatewayPort> OpeningHoursIntentHandler<D> {
    pub fn new(domain_gateway: D) -> Self {
        Self { domain_gateway }
    }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for OpeningHoursIntentHandler<D> {
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
            &input.conversation,
            input.analysis_intent,
            input.text,
            input.analysis_entities,
        );
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply: self.domain_gateway.get_opening_hours(),
            handled_intent: self.intent(),
        }
    }
}
