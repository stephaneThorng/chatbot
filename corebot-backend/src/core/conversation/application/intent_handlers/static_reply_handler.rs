use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::domain::intent::{IntentId, IntentKind, IntentPolicy};

pub struct StaticReplyIntentHandler {
    intent: IntentId,
    reply_key: &'static str,
}

impl StaticReplyIntentHandler {
    pub fn new(intent: IntentId, reply_key: &'static str) -> Self {
        Self { intent, reply_key }
    }
}

impl IntentHandler for StaticReplyIntentHandler {
    fn intent(&self) -> IntentId {
        self.intent.clone()
    }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy {
            id: self.intent(),
            kind: IntentKind::Informational,
            nlu_task: None,
            workflow_slots: vec![],
            supported_entities: vec![],
            confirmation_prompt: None,
            completion_response: None,
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        StateHandlerResult {
            updated_conversation: input.conversation.clone(),
            reply: t!(self.reply_key, locale = input.conversation.lang.as_str()).to_string(),
            handled_intent: self.intent(),
        }
    }
}
