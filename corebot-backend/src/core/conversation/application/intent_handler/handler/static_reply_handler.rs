use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct StaticReplyIntentHandler {
    intent: IntentId,
    reply_key: &'static str,
}

impl StaticReplyIntentHandler {
    pub fn new(intent: IntentId, reply_key: &'static str) -> Self {
        Self { intent, reply_key }
    }
}

#[async_trait::async_trait]
impl IntentHandler for StaticReplyIntentHandler {
    fn intent(&self) -> IntentId {
        self.intent.clone()
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let reply = t!(self.reply_key, locale = input.conversation.lang.as_str()).to_string();
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply: vec![reply],
            handled_intent: self.intent(),
        }
    }
}
