use std::sync::Arc;
use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerInput, StateHandlerResult};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct AskMenuGeneralIntentHandler<D: DomainGatewayPort> {
    domain_gateway: Arc<D>,
}

impl<D: DomainGatewayPort> AskMenuGeneralIntentHandler<D> {
    pub fn new(domain_gateway: Arc<D>) -> Self { Self { domain_gateway } }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for AskMenuGeneralIntentHandler<D> {
    fn intent(&self) -> IntentId { IntentId::AskMenuGeneral }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy { id: self.intent(), kind: IntentKind::Informational, nlu_task: None, workflow_slots: vec![], confirmation_prompt: None, completion_response: None }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let price_item = self.lookup_entity_value(&input, EntityType::PriceItem);
        let comparator = self.lookup_entity_value(&input, EntityType::PriceComparator);
        let amount = self.lookup_entity_value(&input, EntityType::PriceAmount);

        let raw = self.domain_gateway.get_menu(price_item, comparator, amount);
        let reply = parse_menu_reply(&raw, lang, comparator, amount, price_item);

        StateHandlerResult { updated_conversation: input.conversation, reply, handled_intent: self.intent() }
    }
}

fn parse_menu_reply(raw: &str, lang: &str, comparator: Option<&str>, amount: Option<&str>, price_item: Option<&str>) -> String {
    if let Some(payload) = raw.strip_prefix("price_results:") {
        return t!("intent.ask_menu_general.price_results.reply", locale = lang,
            comparator = comparator.unwrap_or(""), amount = amount.unwrap_or(""), items = payload).to_string();
    }
    if raw.starts_with("no_results:") {
        return t!("intent.ask_menu_general.no_results.reply", locale = lang,
            comparator = comparator.unwrap_or(""), amount = amount.unwrap_or("")).to_string();
    }
    if let Some(payload) = raw.strip_prefix("item_found:") {
        let parts: Vec<&str> = payload.splitn(3, '|').collect();
        let name = parts.first().copied().unwrap_or("");
        let price = parts.get(1).copied().unwrap_or("");
        return t!("intent.ask_menu_general.item_found.reply", locale = lang, item = name, price = price).to_string();
    }
    if raw.starts_with("item_not_found:") {
        return t!("intent.ask_menu_general.item_not_found.reply", locale = lang, item = price_item.unwrap_or("")).to_string();
    }
    if let Some(payload) = raw.strip_prefix("full_menu:") {
        return t!("intent.ask_menu_general.full_menu.reply", locale = lang, items = payload).to_string();
    }
    t!("intent.ask_menu_general.reply", locale = lang).to_string()
}

