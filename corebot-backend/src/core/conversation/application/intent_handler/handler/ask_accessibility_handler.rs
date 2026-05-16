use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_accessibility_gateway_port::RestaurantAccessibilityGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskAccessibilityIntentHandler<'a, P: RestaurantAccessibilityGatewayPort + ?Sized> {
    accessibility_gateway_port: &'a P,
}

impl<'a, P: RestaurantAccessibilityGatewayPort + ?Sized> AskAccessibilityIntentHandler<'a, P> {
    pub fn new(accessibility_port: &'a P) -> Self {
        Self {
            accessibility_gateway_port: accessibility_port,
        }
    }
}

#[async_trait::async_trait]
impl<P: RestaurantAccessibilityGatewayPort + Send + Sync + ?Sized> IntentHandler
    for AskAccessibilityIntentHandler<'_, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskAccessibility
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
            .accessibility_gateway_port
            .get_accessibility_info()
            .await;
        let reply = if let Some(info) = raw.strip_prefix("accessibility:yes|") {
            t!(
                "intent.ask_accessibility.confirmed.reply",
                locale = lang,
                info = info
            )
            .to_string()
        } else {
            t!("intent.ask_accessibility.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
