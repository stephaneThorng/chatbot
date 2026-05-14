use rust_i18n::t;
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::application::port::outbound::restaurant_queries::PaymentMethodQuery;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct AskPaymentMethodsIntentHandler<P: RestaurantInformationPort> {
    information_port: Arc<P>,
}

impl<P: RestaurantInformationPort> AskPaymentMethodsIntentHandler<P> {
    pub fn new(information_port: Arc<P>) -> Self {
        Self { information_port }
    }
}

impl<P: RestaurantInformationPort + Send + Sync> IntentHandler
    for AskPaymentMethodsIntentHandler<P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskPaymentMethods
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
        let lang = input.conversation.lang.as_str();
        let method = self.lookup_entity_value(&input, EntityType::PaymentMethod);
        let raw = self
            .information_port
            .find_payment_methods(PaymentMethodQuery {
                method: method.map(str::to_string),
            });
        let reply = if let Some(payload) = raw.strip_prefix("method_accepted:") {
            let mut p = payload.splitn(2, '|');
            let m = p.next().unwrap_or("");
            let all = p.next().unwrap_or("");
            t!(
                "intent.ask_payment_methods.accepted.reply",
                locale = lang,
                method = m,
                all = all
            )
            .to_string()
        } else if let Some(payload) = raw.strip_prefix("method_not_accepted:") {
            let mut p = payload.splitn(2, '|');
            let m = p.next().unwrap_or("");
            let all = p.next().unwrap_or("");
            t!(
                "intent.ask_payment_methods.not_accepted.reply",
                locale = lang,
                method = m,
                all = all
            )
            .to_string()
        } else if let Some(all) = raw.strip_prefix("all_methods:") {
            t!(
                "intent.ask_payment_methods.all.reply",
                locale = lang,
                methods = all
            )
            .to_string()
        } else {
            t!("intent.ask_payment_methods.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
