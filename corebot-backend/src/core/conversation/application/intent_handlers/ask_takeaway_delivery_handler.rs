use std::sync::Arc;
use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerInput, StateHandlerResult};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};

pub struct AskTakeawayDeliveryIntentHandler<D: DomainGatewayPort> {
    domain_gateway: Arc<D>,
}

impl<D: DomainGatewayPort> AskTakeawayDeliveryIntentHandler<D> {
    pub fn new(domain_gateway: Arc<D>) -> Self { Self { domain_gateway } }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for AskTakeawayDeliveryIntentHandler<D> {
    fn intent(&self) -> IntentId { IntentId::AskTakeawayDelivery }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy { id: self.intent(), kind: IntentKind::Informational, nlu_task: None, workflow_slots: vec![], confirmation_prompt: None, completion_response: None }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let raw = self.domain_gateway.get_takeaway_info();
        let reply = if let Some(payload) = raw.strip_prefix("takeaway:yes|") {
            t!("intent.ask_takeaway_delivery.available.reply", locale = lang, info = payload).to_string()
        } else if raw.starts_with("takeaway:no|") {
            t!("intent.ask_takeaway_delivery.unavailable.reply", locale = lang).to_string()
        } else {
            t!("intent.ask_takeaway_delivery.reply", locale = lang).to_string()
        };
        StateHandlerResult { updated_conversation: input.conversation, reply, handled_intent: self.intent() }
    }
}

