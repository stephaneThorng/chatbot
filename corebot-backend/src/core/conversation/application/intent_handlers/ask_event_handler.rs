use std::sync::Arc;
use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerInput, StateHandlerResult};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct AskEventIntentHandler<D: DomainGatewayPort> {
    domain_gateway: Arc<D>,
}

impl<D: DomainGatewayPort> AskEventIntentHandler<D> {
    pub fn new(domain_gateway: Arc<D>) -> Self { Self { domain_gateway } }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for AskEventIntentHandler<D> {
    fn intent(&self) -> IntentId { IntentId::AskEvent }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy { id: self.intent(), kind: IntentKind::Informational, nlu_task: None, workflow_slots: vec![], confirmation_prompt: None, completion_response: None }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let location = self.lookup_entity_value(&input, EntityType::Location);
        let raw = self.domain_gateway.get_event_info(location);
        let reply = if let Some(payload) = raw.strip_prefix("event_space_available:") {
            let mut p = payload.splitn(2, '|');
            let loc = p.next().unwrap_or("");
            let info = p.next().unwrap_or("");
            t!("intent.ask_event.space_available.reply", locale = lang, location = loc, info = info).to_string()
        } else if let Some(payload) = raw.strip_prefix("event_space_unavailable:") {
            let mut p = payload.splitn(2, '|');
            let loc = p.next().unwrap_or("");
            let info = p.next().unwrap_or("");
            t!("intent.ask_event.space_unavailable.reply", locale = lang, location = loc, info = info).to_string()
        } else if let Some(info) = raw.strip_prefix("event_info:") {
            t!("intent.ask_event.info.reply", locale = lang, info = info).to_string()
        } else {
            t!("intent.ask_event.reply", locale = lang).to_string()
        };
        StateHandlerResult { updated_conversation: input.conversation, reply, handled_intent: self.intent() }
    }
}

