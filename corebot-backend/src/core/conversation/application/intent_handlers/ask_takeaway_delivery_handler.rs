use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};

pub struct AskTakeawayDeliveryIntentHandler<'a, P: RestaurantInformationPort + ?Sized> {
    information_port: &'a P,
}

impl<'a, P: RestaurantInformationPort + ?Sized> AskTakeawayDeliveryIntentHandler<'a, P> {
    pub fn new(information_port: &'a P) -> Self {
        Self { information_port }
    }
}

impl<'a, P: RestaurantInformationPort + Send + Sync + ?Sized> IntentHandler
    for AskTakeawayDeliveryIntentHandler<'a, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskTakeawayDelivery
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
        let raw = self.information_port.get_takeaway_info();
        let reply = if let Some(payload) = raw.strip_prefix("takeaway:yes|") {
            t!(
                "intent.ask_takeaway_delivery.available.reply",
                locale = lang,
                info = payload
            )
            .to_string()
        } else if raw.starts_with("takeaway:no|") {
            t!(
                "intent.ask_takeaway_delivery.unavailable.reply",
                locale = lang
            )
            .to_string()
        } else {
            t!("intent.ask_takeaway_delivery.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
