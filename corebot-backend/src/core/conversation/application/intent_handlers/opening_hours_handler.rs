use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};

pub struct OpeningHoursIntentHandler<'a, P: RestaurantInformationPort + ?Sized> {
    information_port: &'a P,
}

impl<'a, P: RestaurantInformationPort + ?Sized> OpeningHoursIntentHandler<'a, P> {
    pub fn new(information_port: &'a P) -> Self {
        Self { information_port }
    }
}

impl<'a, P: RestaurantInformationPort + Send + Sync + ?Sized> IntentHandler
    for OpeningHoursIntentHandler<'a, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskOpeningHours
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
