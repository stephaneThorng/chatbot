use rust_i18n::t;
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::application::port::outbound::restaurant_queries::FacilityQuery;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct AskFacilitiesIntentHandler<P: RestaurantInformationPort> {
    information_port: Arc<P>,
}

impl<P: RestaurantInformationPort> AskFacilitiesIntentHandler<P> {
    pub fn new(information_port: Arc<P>) -> Self {
        Self { information_port }
    }
}

impl<P: RestaurantInformationPort + Send + Sync> IntentHandler for AskFacilitiesIntentHandler<P> {
    fn intent(&self) -> IntentId {
        IntentId::AskFacilities
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
        let facility = self.lookup_entity_value(&input, EntityType::Facility);
        let raw = self.information_port.find_facility_info(FacilityQuery {
            facility: facility.map(str::to_string),
        });
        let reply = if let Some(f) = raw.strip_prefix("facility_available:") {
            t!(
                "intent.ask_facilities.available.reply",
                locale = lang,
                facility = f
            )
            .to_string()
        } else if let Some(f) = raw.strip_prefix("facility_unavailable:") {
            t!(
                "intent.ask_facilities.unavailable.reply",
                locale = lang,
                facility = f
            )
            .to_string()
        } else if let Some(all) = raw.strip_prefix("all_facilities:") {
            t!(
                "intent.ask_facilities.all.reply",
                locale = lang,
                facilities = all
            )
            .to_string()
        } else {
            t!("intent.ask_facilities.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
