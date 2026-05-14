use rust_i18n::t;
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::application::port::outbound::restaurant_queries::EventQuery;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};


pub struct AskEventIntentHandler<P: RestaurantInformationPort> {
    information_port: Arc<P>,
}

impl<P: RestaurantInformationPort> AskEventIntentHandler<P> {
    pub fn new(information_port: Arc<P>) -> Self {
        Self { information_port }
    }
}

impl<P: RestaurantInformationPort + Send + Sync> IntentHandler for AskEventIntentHandler<P> {
    fn intent(&self) -> IntentId {
        IntentId::AskEvent
    }

    fn config(&self) -> IntentConfig {
        IntentConfig { id: self.intent(), workflow: IntentWorkflow::Informational }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let location = self.lookup_entity_value(&input, "location");
        let raw = self.information_port.find_event_info(EventQuery {
            location: location.map(str::to_string),
        });
        let reply = if let Some(payload) = raw.strip_prefix("event_space_available:") {
            let mut p = payload.splitn(2, '|');
            let loc = p.next().unwrap_or("");
            let info = p.next().unwrap_or("");
            t!(
                "intent.ask_event.space_available.reply",
                locale = lang,
                location = loc,
                info = info
            )
            .to_string()
        } else if let Some(payload) = raw.strip_prefix("event_space_unavailable:") {
            let mut p = payload.splitn(2, '|');
            let loc = p.next().unwrap_or("");
            let info = p.next().unwrap_or("");
            t!(
                "intent.ask_event.space_unavailable.reply",
                locale = lang,
                location = loc,
                info = info
            )
            .to_string()
        } else if let Some(info) = raw.strip_prefix("event_info:") {
            t!("intent.ask_event.info.reply", locale = lang, info = info).to_string()
        } else {
            t!("intent.ask_event.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
