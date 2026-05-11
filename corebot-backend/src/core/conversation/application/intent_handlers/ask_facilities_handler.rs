use std::sync::Arc;
use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerInput, StateHandlerResult};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct AskFacilitiesIntentHandler<D: DomainGatewayPort> {
    domain_gateway: Arc<D>,
}

impl<D: DomainGatewayPort> AskFacilitiesIntentHandler<D> {
    pub fn new(domain_gateway: Arc<D>) -> Self { Self { domain_gateway } }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for AskFacilitiesIntentHandler<D> {
    fn intent(&self) -> IntentId { IntentId::AskFacilities }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy { id: self.intent(), kind: IntentKind::Informational, nlu_task: None, workflow_slots: vec![], confirmation_prompt: None, completion_response: None }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let facility = self.lookup_entity_value(&input, EntityType::Facility);
        let raw = self.domain_gateway.get_facility_info(facility);
        let reply = if let Some(f) = raw.strip_prefix("facility_available:") {
            t!("intent.ask_facilities.available.reply", locale = lang, facility = f).to_string()
        } else if let Some(f) = raw.strip_prefix("facility_unavailable:") {
            t!("intent.ask_facilities.unavailable.reply", locale = lang, facility = f).to_string()
        } else if let Some(all) = raw.strip_prefix("all_facilities:") {
            t!("intent.ask_facilities.all.reply", locale = lang, facilities = all).to_string()
        } else {
            t!("intent.ask_facilities.reply", locale = lang).to_string()
        };
        StateHandlerResult { updated_conversation: input.conversation, reply, handled_intent: self.intent() }
    }
}

