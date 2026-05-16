use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_takeaway_gateway_port::RestaurantTakeawayGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskTakeawayDeliveryIntentHandler<'a, P: RestaurantTakeawayGatewayPort + ?Sized> {
    takeaway_gateway_port: &'a P,
}

impl<'a, P: RestaurantTakeawayGatewayPort + ?Sized> AskTakeawayDeliveryIntentHandler<'a, P> {
    pub fn new(takeaway_port: &'a P) -> Self {
        Self {
            takeaway_gateway_port: takeaway_port,
        }
    }
}

#[async_trait::async_trait]
impl<P: RestaurantTakeawayGatewayPort + Send + Sync + ?Sized> IntentHandler
    for AskTakeawayDeliveryIntentHandler<'_, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskTakeawayDelivery
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let raw = self.takeaway_gateway_port.get_takeaway_info().await;
        let reply = if let Some(payload) = raw.strip_prefix("takeaway:yes|") {
            t!(
                "intent.ask_takeaway_delivery.available.reply",
                locale = lang,
                info = payload
            )
            .to_string()
        } else if raw.starts_with("takeaway:no|") {
            t!(
                "intent.ask_takeaway_delivery.unavailable.reply",
                locale = lang
            )
            .to_string()
        } else {
            t!("intent.ask_takeaway_delivery.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
