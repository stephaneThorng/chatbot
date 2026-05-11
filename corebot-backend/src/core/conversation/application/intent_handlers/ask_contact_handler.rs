use std::sync::Arc;
use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerInput, StateHandlerResult};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};

pub struct AskContactIntentHandler<D: DomainGatewayPort> {
    domain_gateway: Arc<D>,
}

impl<D: DomainGatewayPort> AskContactIntentHandler<D> {
    pub fn new(domain_gateway: Arc<D>) -> Self { Self { domain_gateway } }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for AskContactIntentHandler<D> {
    fn intent(&self) -> IntentId { IntentId::AskContact }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy { id: self.intent(), kind: IntentKind::Informational, nlu_task: None, workflow_slots: vec![], confirmation_prompt: None, completion_response: None }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let raw = self.domain_gateway.get_contact();
        let reply = if let Some(payload) = raw.strip_prefix("contact:") {
            let mut p = payload.splitn(2, '|');
            let phone = p.next().unwrap_or("");
            let email = p.next().unwrap_or("");
            t!("intent.ask_contact.data.reply", locale = lang, phone = phone, email = email).to_string()
        } else {
            t!("intent.ask_contact.reply", locale = lang).to_string()
        };
        StateHandlerResult { updated_conversation: input.conversation, reply, handled_intent: self.intent() }
    }
}

