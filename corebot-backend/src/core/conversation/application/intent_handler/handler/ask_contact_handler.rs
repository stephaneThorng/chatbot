use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_contact_gateway_port::RestaurantContactGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskContactIntentHandler<'a, P: RestaurantContactGatewayPort + ?Sized> {
    contact_gateway_port: &'a P,
}

impl<'a, P: RestaurantContactGatewayPort + ?Sized> AskContactIntentHandler<'a, P> {
    pub fn new(contact_port: &'a P) -> Self {
        Self {
            contact_gateway_port: contact_port,
        }
    }
}

#[async_trait::async_trait]
impl<P: RestaurantContactGatewayPort + Send + Sync + ?Sized> IntentHandler
    for AskContactIntentHandler<'_, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskContact
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let raw = self.contact_gateway_port.get_contact().await;
        let reply = if let Some(payload) = raw.strip_prefix("contact:") {
            let mut p = payload.splitn(2, '|');
            let phone = p.next().unwrap_or("");
            let email = p.next().unwrap_or("");
            t!(
                "intent.ask_contact.data.reply",
                locale = lang,
                phone = phone,
                email = email
            )
            .to_string()
        } else {
            t!("intent.ask_contact.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
