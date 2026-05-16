use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::business_info_queries::PaymentMethodQuery;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_payment_methods_gateway_port::RestaurantPaymentMethodsGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskPaymentMethodsIntentHandler<'a, P: RestaurantPaymentMethodsGatewayPort + ?Sized> {
    payment_methods_gateway_port: &'a P,
}

impl<'a, P: RestaurantPaymentMethodsGatewayPort + ?Sized> AskPaymentMethodsIntentHandler<'a, P> {
    pub fn new(payment_methods_port: &'a P) -> Self {
        Self {
            payment_methods_gateway_port: payment_methods_port,
        }
    }
}

#[async_trait::async_trait]
impl<P: RestaurantPaymentMethodsGatewayPort + Send + Sync + ?Sized> IntentHandler
    for AskPaymentMethodsIntentHandler<'_, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskPaymentMethods
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let method = self.lookup_entity_value(&input, "payment_method");
        let raw = self
            .payment_methods_gateway_port
            .find_payment_methods(PaymentMethodQuery {
                method: method.map(str::to_string),
            })
            .await;
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
