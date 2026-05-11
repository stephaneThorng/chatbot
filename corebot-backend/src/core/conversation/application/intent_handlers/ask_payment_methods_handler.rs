use std::sync::Arc;
use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerInput, StateHandlerResult};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct AskPaymentMethodsIntentHandler<D: DomainGatewayPort> {
    domain_gateway: Arc<D>,
}

impl<D: DomainGatewayPort> AskPaymentMethodsIntentHandler<D> {
    pub fn new(domain_gateway: Arc<D>) -> Self { Self { domain_gateway } }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for AskPaymentMethodsIntentHandler<D> {
    fn intent(&self) -> IntentId { IntentId::AskPaymentMethods }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy { id: self.intent(), kind: IntentKind::Informational, nlu_task: None, workflow_slots: vec![], confirmation_prompt: None, completion_response: None }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let method = self.lookup_entity_value(&input, EntityType::PaymentMethod);
        let raw = self.domain_gateway.get_payment_methods(method);
        let reply = if let Some(payload) = raw.strip_prefix("method_accepted:") {
            let mut p = payload.splitn(2, '|');
            let m = p.next().unwrap_or("");
            let all = p.next().unwrap_or("");
            t!("intent.ask_payment_methods.accepted.reply", locale = lang, method = m, all = all).to_string()
        } else if let Some(payload) = raw.strip_prefix("method_not_accepted:") {
            let mut p = payload.splitn(2, '|');
            let m = p.next().unwrap_or("");
            let all = p.next().unwrap_or("");
            t!("intent.ask_payment_methods.not_accepted.reply", locale = lang, method = m, all = all).to_string()
        } else if let Some(all) = raw.strip_prefix("all_methods:") {
            t!("intent.ask_payment_methods.all.reply", locale = lang, methods = all).to_string()
        } else {
            t!("intent.ask_payment_methods.reply", locale = lang).to_string()
        };
        StateHandlerResult { updated_conversation: input.conversation, reply, handled_intent: self.intent() }
    }
}

