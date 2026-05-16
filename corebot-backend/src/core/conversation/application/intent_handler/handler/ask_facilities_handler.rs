use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::business_info_queries::FacilityQuery;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_facilities_gateway_port::RestaurantFacilitiesGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};
use rust_i18n::t;

pub struct AskFacilitiesIntentHandler<'a, P: RestaurantFacilitiesGatewayPort + ?Sized> {
    facilities_gateway_port: &'a P,
}

impl<'a, P: RestaurantFacilitiesGatewayPort + ?Sized> AskFacilitiesIntentHandler<'a, P> {
    pub fn new(facilities_port: &'a P) -> Self {
        Self {
            facilities_gateway_port: facilities_port,
        }
    }
}

#[async_trait::async_trait]
impl<P: RestaurantFacilitiesGatewayPort + Send + Sync + ?Sized> IntentHandler
    for AskFacilitiesIntentHandler<'_, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskFacilities
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let facility = self.lookup_entity_value(&input, "facility");
        let raw = self
            .facilities_gateway_port
            .find_facility_info(FacilityQuery {
                facility: facility.map(str::to_string),
            })
            .await;
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
