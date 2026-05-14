use rust_i18n::t;
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};

pub struct AskEntertainmentIntentHandler<P: RestaurantInformationPort> {
    information_port: Arc<P>,
}

impl<P: RestaurantInformationPort> AskEntertainmentIntentHandler<P> {
    pub fn new(information_port: Arc<P>) -> Self {
        Self { information_port }
    }
}

impl<P: RestaurantInformationPort + Send + Sync> IntentHandler
    for AskEntertainmentIntentHandler<P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskEntertainment
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
        let raw = self.information_port.get_entertainment_info();
        let reply = if let Some(info) = raw.strip_prefix("entertainment:yes|") {
            t!(
                "intent.ask_entertainment.confirmed.reply",
                locale = lang,
                info = info
            )
            .to_string()
        } else {
            t!("intent.ask_entertainment.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
