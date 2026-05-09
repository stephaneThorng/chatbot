use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, IntentHandlerResult,
};
use crate::core::conversation::application::port::outbound::domain_gateway_trait::DomainGatewayPort;
use crate::core::conversation::domain::intent::IntentId;

pub struct OpeningHoursIntentHandler {
    domain_gateway: Arc<dyn DomainGatewayPort>,
}

impl OpeningHoursIntentHandler {
    pub fn new(domain_gateway: Arc<dyn DomainGatewayPort>) -> Self {
        Self { domain_gateway }
    }
}

impl IntentHandler for OpeningHoursIntentHandler {
    fn intent(&self) -> IntentId {
        IntentId::new("ask_opening_hours")
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> IntentHandlerResult {
        let _ = (input.intent, input.text, input.lang, input.entities);
        IntentHandlerResult {
            reply: self.domain_gateway.get_opening_hours(),
        }
    }
}
