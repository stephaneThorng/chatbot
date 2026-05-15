use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::application::port::outbound::restaurant_queries::EventQuery;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct AskEventIntentHandler<'a, P: RestaurantInformationPort + ?Sized> {
    information_port: &'a P,
}

impl<'a, P: RestaurantInformationPort + ?Sized> AskEventIntentHandler<'a, P> {
    pub fn new(information_port: &'a P) -> Self {
        Self { information_port }
    }
}

impl<'a, P: RestaurantInformationPort + Send + Sync + ?Sized> IntentHandler
    for AskEventIntentHandler<'a, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskEvent
    }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy {
            id: self.intent(),
            kind: IntentKind::Informational,
            nlu_task: None,
            workflow_slots: vec![],
            starting_message: None,
            confirmation_prompt: None,
            completion_response: None,
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let location = self.lookup_entity_value(&input, EntityType::Location);
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
