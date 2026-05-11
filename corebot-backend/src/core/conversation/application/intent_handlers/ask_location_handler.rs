use std::sync::Arc;
use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerInput, StateHandlerResult};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct AskLocationIntentHandler<D: DomainGatewayPort> {
    domain_gateway: Arc<D>,
}

impl<D: DomainGatewayPort> AskLocationIntentHandler<D> {
    pub fn new(domain_gateway: Arc<D>) -> Self { Self { domain_gateway } }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for AskLocationIntentHandler<D> {
    fn intent(&self) -> IntentId { IntentId::AskLocation }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy { id: self.intent(), kind: IntentKind::Informational, nlu_task: None, workflow_slots: vec![], confirmation_prompt: None, completion_response: None }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let near = self.lookup_entity_value(&input, EntityType::Location);
        let raw = self.domain_gateway.get_location(near);
        let reply = if let Some(payload) = raw.strip_prefix("near_confirmed:") {
            let mut p = payload.splitn(2, '|');
            let loc = p.next().unwrap_or("");
            let addr = p.next().unwrap_or("");
            t!("intent.ask_location.near_confirmed.reply", locale = lang, location = loc, address = addr).to_string()
        } else if let Some(payload) = raw.strip_prefix("near_denied:") {
            let mut p = payload.splitn(2, '|');
            let loc = p.next().unwrap_or("");
            let addr = p.next().unwrap_or("");
            t!("intent.ask_location.near_denied.reply", locale = lang, location = loc, address = addr).to_string()
        } else if let Some(addr) = raw.strip_prefix("address:") {
            t!("intent.ask_location.address.reply", locale = lang, address = addr).to_string()
        } else {
            t!("intent.ask_location.reply", locale = lang).to_string()
        };
        StateHandlerResult { updated_conversation: input.conversation, reply, handled_intent: self.intent() }
    }
}

