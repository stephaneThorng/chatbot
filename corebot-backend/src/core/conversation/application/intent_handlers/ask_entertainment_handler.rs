use std::sync::Arc;
use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerInput, StateHandlerResult};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};

pub struct AskEntertainmentIntentHandler<D: DomainGatewayPort> {
    domain_gateway: Arc<D>,
}

impl<D: DomainGatewayPort> AskEntertainmentIntentHandler<D> {
    pub fn new(domain_gateway: Arc<D>) -> Self { Self { domain_gateway } }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for AskEntertainmentIntentHandler<D> {
    fn intent(&self) -> IntentId { IntentId::AskEntertainment }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy { id: self.intent(), kind: IntentKind::Informational, nlu_task: None, workflow_slots: vec![], confirmation_prompt: None, completion_response: None }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let raw = self.domain_gateway.get_entertainment_info();
        let reply = if let Some(info) = raw.strip_prefix("entertainment:yes|") {
            t!("intent.ask_entertainment.confirmed.reply", locale = lang, info = info).to_string()
        } else {
            t!("intent.ask_entertainment.reply", locale = lang).to_string()
        };
        StateHandlerResult { updated_conversation: input.conversation, reply, handled_intent: self.intent() }
    }
}

