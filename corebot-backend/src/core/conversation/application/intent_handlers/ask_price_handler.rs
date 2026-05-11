use std::sync::Arc;
use rust_i18n::t;

use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerInput, StateHandlerResult};
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct AskPriceIntentHandler<D: DomainGatewayPort> {
    domain_gateway: Arc<D>,
}

impl<D: DomainGatewayPort> AskPriceIntentHandler<D> {
    pub fn new(domain_gateway: Arc<D>) -> Self { Self { domain_gateway } }
}

impl<D: DomainGatewayPort + Send + Sync> IntentHandler for AskPriceIntentHandler<D> {
    fn intent(&self) -> IntentId { IntentId::AskPrice }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy { id: self.intent(), kind: IntentKind::Informational, nlu_task: None, workflow_slots: vec![], confirmation_prompt: None, completion_response: None }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let price_item = self.lookup_entity_value(&input, EntityType::PriceItem);
        let menu_item = self.lookup_entity_value(&input, EntityType::MenuItem);
        let comparator = self.lookup_entity_value(&input, EntityType::PriceComparator);
        let amount = self.lookup_entity_value(&input, EntityType::PriceAmount);
        let item = price_item.or(menu_item);
        let raw = self.domain_gateway.get_price(item, comparator, amount);

        let reply = if let Some(payload) = raw.strip_prefix("price_results:") {
            let parts: Vec<&str> = payload.splitn(3, '|').collect();
            let comp = parts.first().copied().unwrap_or("");
            let amt = parts.get(1).copied().unwrap_or("");
            let items = parts.get(2).copied().unwrap_or("");
            t!("intent.ask_price.results.reply", locale = lang, comparator = comp, amount = amt, items = items).to_string()
        } else if let Some(payload) = raw.strip_prefix("item_price:") {
            let mut p = payload.splitn(2, '|');
            let name = p.next().unwrap_or("");
            let price = p.next().unwrap_or("");
            t!("intent.ask_price.item_price.reply", locale = lang, item = name, price = price).to_string()
        } else if raw.starts_with("no_price_results:") || raw.starts_with("item_not_found:") {
            t!("intent.ask_price.no_results.reply", locale = lang, item = item.unwrap_or("")).to_string()
        } else if let Some(info) = raw.strip_prefix("price_general:") {
            t!("intent.ask_price.general.reply", locale = lang, info = info).to_string()
        } else {
            t!("intent.ask_price.reply", locale = lang).to_string()
        };
        StateHandlerResult { updated_conversation: input.conversation, reply, handled_intent: self.intent() }
    }
}

