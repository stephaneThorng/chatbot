use std::sync::Arc;
use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerInput, StateHandlerResult};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct AskMenuDietaryIntentHandler<D: DomainGatewayPort> {
    domain_gateway: Arc<D>,
}

impl<D: DomainGatewayPort> AskMenuDietaryIntentHandler<D> {
    pub fn new(domain_gateway: Arc<D>) -> Self { Self { domain_gateway } }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for AskMenuDietaryIntentHandler<D> {
    fn intent(&self) -> IntentId { IntentId::AskMenuDietary }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy { id: self.intent(), kind: IntentKind::Informational, nlu_task: None, workflow_slots: vec![], confirmation_prompt: None, completion_response: None }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let dietary = self.lookup_entity_value(&input, EntityType::DietaryRequirement);
        let raw = self.domain_gateway.get_menu_dietary(dietary);
        let reply = parse_dietary_reply(&raw, lang, dietary);
        StateHandlerResult { updated_conversation: input.conversation, reply, handled_intent: self.intent() }
    }
}

fn parse_dietary_reply(raw: &str, lang: &str, dietary: Option<&str>) -> String {
    if let Some(payload) = raw.strip_prefix("dietary_results:") {
        let mut parts = payload.splitn(2, '|');
        let req = parts.next().unwrap_or("");
        let items = parts.next().unwrap_or("");
        return t!("intent.ask_menu_dietary.results.reply", locale = lang, requirement = req, items = items).to_string();
    }
    if let Some(req) = raw.strip_prefix("no_dietary:") {
        return t!("intent.ask_menu_dietary.no_results.reply", locale = lang, requirement = req).to_string();
    }
    if let Some(options) = raw.strip_prefix("dietary_no_filter:") {
        return t!("intent.ask_menu_dietary.options.reply", locale = lang, options = options).to_string();
    }
    t!("intent.ask_menu_dietary.reply", locale = lang).to_string()
}

