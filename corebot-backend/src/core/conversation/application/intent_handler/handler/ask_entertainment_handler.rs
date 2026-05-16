use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_entertainment_gateway_port::RestaurantEntertainmentGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskEntertainmentIntentHandler<'a, P: RestaurantEntertainmentGatewayPort + ?Sized> {
    entertainment_gateway_port: &'a P,
}

impl<'a, P: RestaurantEntertainmentGatewayPort + ?Sized> AskEntertainmentIntentHandler<'a, P> {
    pub fn new(entertainment_port: &'a P) -> Self {
        Self {
            entertainment_gateway_port: entertainment_port,
        }
    }
}

#[async_trait::async_trait]
impl<P: RestaurantEntertainmentGatewayPort + Send + Sync + ?Sized> IntentHandler
    for AskEntertainmentIntentHandler<'_, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskEntertainment
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let raw = self
            .entertainment_gateway_port
            .get_entertainment_info()
            .await;
        let reply = if let Some(info) = raw.strip_prefix("entertainment:yes|") {
            t!(
                "intent.ask_entertainment.confirmed.reply",
                locale = lang,
                info = info
            )
            .to_string()
        } else {
            t!("intent.ask_entertainment.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
