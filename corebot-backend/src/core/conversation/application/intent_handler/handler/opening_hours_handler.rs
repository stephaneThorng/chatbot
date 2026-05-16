use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_opening_hours_gateway_port::RestaurantOpeningHoursGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct OpeningHoursIntentHandler<'a, P: RestaurantOpeningHoursGatewayPort + ?Sized> {
    opening_hours_gateway_port: &'a P,
}

impl<'a, P: RestaurantOpeningHoursGatewayPort + ?Sized> OpeningHoursIntentHandler<'a, P> {
    pub fn new(opening_hours_port: &'a P) -> Self {
        Self {
            opening_hours_gateway_port: opening_hours_port,
        }
    }
}

#[async_trait::async_trait]
impl<P: RestaurantOpeningHoursGatewayPort + Send + Sync + ?Sized> IntentHandler
    for OpeningHoursIntentHandler<'_, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskOpeningHours
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let _ = (
            &input.conversation,
            input.analysis_intent,
            input.text,
            input.analysis_entities,
        );
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply: self.opening_hours_gateway_port.get_opening_hours().await,
            handled_intent: self.intent(),
        }
    }
}
