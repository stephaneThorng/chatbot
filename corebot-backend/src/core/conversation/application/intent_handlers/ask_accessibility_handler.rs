use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};

pub struct AskAccessibilityIntentHandler<'a, P: RestaurantInformationPort + ?Sized> {
    information_port: &'a P,
}

impl<'a, P: RestaurantInformationPort + ?Sized> AskAccessibilityIntentHandler<'a, P> {
    pub fn new(information_port: &'a P) -> Self {
        Self { information_port }
    }
}

impl<'a, P: RestaurantInformationPort + Send + Sync + ?Sized> IntentHandler
    for AskAccessibilityIntentHandler<'a, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskAccessibility
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
        let raw = self.information_port.get_accessibility_info();
        let reply = if let Some(info) = raw.strip_prefix("accessibility:yes|") {
            t!(
                "intent.ask_accessibility.confirmed.reply",
                locale = lang,
                info = info
            )
            .to_string()
        } else {
            t!("intent.ask_accessibility.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
