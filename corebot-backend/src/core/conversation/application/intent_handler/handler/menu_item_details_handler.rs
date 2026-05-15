use rust_i18n::t;
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::application::port::outbound::restaurant_queries::MenuItemDetailsQuery;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct MenuItemDetailsIntentHandler<P: RestaurantInformationPort> {
    information_port: Arc<P>,
}

impl<P: RestaurantInformationPort> MenuItemDetailsIntentHandler<P> {
    pub fn new(information_port: Arc<P>) -> Self {
        Self { information_port }
    }
}

impl<P: RestaurantInformationPort + Send + Sync> IntentHandler for MenuItemDetailsIntentHandler<P> {
    fn intent(&self) -> IntentId {
        IntentId::AskMenuItemDetails
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let menu_item = self.lookup_entity_value(&input, "menu_item");
        let allergen = self.lookup_entity_value(&input, "allergen");
        let raw = self
            .information_port
            .find_menu_item_details(MenuItemDetailsQuery {
                menu_item: menu_item.map(str::to_string),
                allergen: allergen.map(str::to_string),
            });
        let reply = parse_item_details_reply(&raw, lang);
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}

fn parse_item_details_reply(raw: &str, lang: &str) -> String {
    if let Some(payload) = raw.strip_prefix("contains:") {
        let mut parts = payload.splitn(2, '|');
        let item = parts.next().unwrap_or("");
        let allergen = parts.next().unwrap_or("");
        return t!(
            "intent.ask_menu_item_details.contains.reply",
            locale = lang,
            item = item,
            allergen = allergen
        )
        .to_string();
    }
    if let Some(payload) = raw.strip_prefix("not_contains:") {
        let mut parts = payload.splitn(2, '|');
        let item = parts.next().unwrap_or("");
        let allergen = parts.next().unwrap_or("");
        return t!(
            "intent.ask_menu_item_details.not_contains.reply",
            locale = lang,
            item = item,
            allergen = allergen
        )
        .to_string();
    }
    if let Some(payload) = raw.strip_prefix("item_details:") {
        let parts: Vec<&str> = payload.splitn(4, '|').collect();
        let name = parts.first().copied().unwrap_or("");
        let price = parts.get(1).copied().unwrap_or("");
        let dietary = parts.get(2).copied().unwrap_or("");
        let allergens = parts.get(3).copied().unwrap_or("");
        return t!(
            "intent.ask_menu_item_details.full.reply",
            locale = lang,
            item = name,
            price = price,
            dietary = dietary,
            allergens = allergens
        )
        .to_string();
    }
    if let Some(payload) = raw.strip_prefix("allergen_found:") {
        let mut parts = payload.splitn(2, '|');
        let allergen = parts.next().unwrap_or("");
        let items = parts.next().unwrap_or("");
        return t!(
            "intent.ask_menu_item_details.allergen_found.reply",
            locale = lang,
            allergen = allergen,
            items = items
        )
        .to_string();
    }
    if raw.starts_with("no_allergen_match:") || raw.starts_with("item_unknown:") {
        return t!(
            "intent.ask_menu_item_details.not_found.reply",
            locale = lang
        )
        .to_string();
    }
    t!("intent.ask_menu_item_details.reply", locale = lang).to_string()
}
