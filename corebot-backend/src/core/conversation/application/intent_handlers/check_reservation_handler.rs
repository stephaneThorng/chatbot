use std::sync::Arc;
use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerInput, StateHandlerResult};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct CheckReservationIntentHandler<D: DomainGatewayPort> {
    domain_gateway: Arc<D>,
}

impl<D: DomainGatewayPort> CheckReservationIntentHandler<D> {
    pub fn new(domain_gateway: Arc<D>) -> Self { Self { domain_gateway } }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for CheckReservationIntentHandler<D> {
    fn intent(&self) -> IntentId { IntentId::CheckReservation }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy { id: self.intent(), kind: IntentKind::Informational, nlu_task: None, workflow_slots: vec![], confirmation_prompt: None, completion_response: None }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let reference = self.lookup_entity_value(&input, EntityType::ReservationReference);
        let raw = self.domain_gateway.check_reservation(reference);

        let reply = if let Some(payload) = raw.strip_prefix("found:") {
            let parts: Vec<&str> = payload.splitn(4, '|').collect();
            let r#ref = parts.first().copied().unwrap_or("");
            let name = parts.get(1).copied().unwrap_or("");
            let date = parts.get(2).copied().unwrap_or("");
            let people = parts.get(3).copied().unwrap_or("");
            t!("intent.check_reservation.found.reply", locale = lang, reference = r#ref, name = name, date = date, people = people).to_string()
        } else if let Some(r) = raw.strip_prefix("not_found:") {
            t!("intent.check_reservation.not_found.reply", locale = lang, reference = r).to_string()
        } else {
            t!("intent.check_reservation.reply", locale = lang).to_string()
        };

        StateHandlerResult { updated_conversation: input.conversation, reply, handled_intent: self.intent() }
    }
}

