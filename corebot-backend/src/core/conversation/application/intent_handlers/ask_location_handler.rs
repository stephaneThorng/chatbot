use rust_i18n::t;
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::application::port::outbound::restaurant_queries::LocationQuery;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskLocationIntentHandler<P: RestaurantInformationPort> {
    information_port: Arc<P>,
}

impl<P: RestaurantInformationPort> AskLocationIntentHandler<P> {
    pub fn new(information_port: Arc<P>) -> Self {
        Self { information_port }
    }
}

impl<P: RestaurantInformationPort + Send + Sync> IntentHandler for AskLocationIntentHandler<P> {
    fn intent(&self) -> IntentId {
        IntentId::AskLocation
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let near = self.lookup_entity_value(&input, "location");
        let raw = self.information_port.find_location(LocationQuery {
            near: near.map(str::to_string),
        });
        let reply = if let Some(payload) = raw.strip_prefix("near_confirmed:") {
            let mut p = payload.splitn(2, '|');
            let loc = p.next().unwrap_or("");
            let addr = p.next().unwrap_or("");
            t!(
                "intent.ask_location.near_confirmed.reply",
                locale = lang,
                location = loc,
                address = addr
            )
            .to_string()
        } else if let Some(payload) = raw.strip_prefix("near_denied:") {
            let mut p = payload.splitn(2, '|');
            let loc = p.next().unwrap_or("");
            let addr = p.next().unwrap_or("");
            t!(
                "intent.ask_location.near_denied.reply",
                locale = lang,
                location = loc,
                address = addr
            )
            .to_string()
        } else if let Some(addr) = raw.strip_prefix("address:") {
            t!(
                "intent.ask_location.address.reply",
                locale = lang,
                address = addr
            )
            .to_string()
        } else {
            t!("intent.ask_location.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
