use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct OpeningHoursIntentHandler<P: RestaurantInformationPort + ?Sized> {
    information_port: std::sync::Arc<P>,
}

impl<P: RestaurantInformationPort + ?Sized> OpeningHoursIntentHandler<P> {
    pub fn new(information_port: std::sync::Arc<P>) -> Self {
        Self { information_port }
    }
}

impl<P: RestaurantInformationPort + Send + Sync + ?Sized> IntentHandler
    for OpeningHoursIntentHandler<P>
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

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let _ = (
            &input.conversation,
            input.analysis_intent,
            input.text,
            input.analysis_entities,
        );
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply: self.information_port.get_opening_hours(),
            handled_intent: self.intent(),
        }
    }
}
