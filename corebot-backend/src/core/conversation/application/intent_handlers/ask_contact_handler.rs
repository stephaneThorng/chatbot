use rust_i18n::t;
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskContactIntentHandler<P: RestaurantInformationPort + ?Sized> {
    information_port: Arc<P>,
}

impl<P: RestaurantInformationPort + ?Sized> AskContactIntentHandler<P> {
    pub fn new(information_port: Arc<P>) -> Self {
        Self { information_port }
    }
}

impl<P: RestaurantInformationPort + Send + Sync + ?Sized> IntentHandler
    for AskContactIntentHandler<P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskContact
    }

    fn config(&self) -> IntentConfig {
        IntentConfig { id: self.intent(), workflow: IntentWorkflow::Informational }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let raw = self.information_port.get_contact();
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
