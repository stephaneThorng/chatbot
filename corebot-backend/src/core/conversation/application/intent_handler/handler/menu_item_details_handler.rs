use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::menu_queries::MenuItemDetailsQuery;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::conversation::application::service::restaurant::{
    ConversationRestaurantMenuService,
};
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct MenuItemDetailsIntentHandler<'a, M> {
    menu_service: &'a ConversationRestaurantMenuService<M>,
}

impl<'a, M> MenuItemDetailsIntentHandler<'a, M> {
    pub fn new(menu_service: &'a ConversationRestaurantMenuService<M>) -> Self {
        Self { menu_service }
    }
}

#[async_trait::async_trait]
impl<M> IntentHandler for MenuItemDetailsIntentHandler<'_, M>
where
    M: RestaurantMenuRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::AskMenuItemDetails
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let menu_item = self.lookup_entity_value(&input, "menu_item");
        let allergen = self.lookup_entity_value(&input, "allergen");
        let raw = self
            .menu_service
            .find_menu_item_details(
                input.conversation.business_id,
                lang,
                MenuItemDetailsQuery {
                    menu_item: menu_item.map(str::to_string),
                    allergen: allergen.map(str::to_string),
                },
            )
            .await;
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
        let parts: Vec<&str> = payload.splitn(5, '|').collect();
        let name = parts.first().copied().unwrap_or("");
        let price = parts.get(1).copied().unwrap_or("");
        let ingredients = parts.get(2).copied().unwrap_or("");
        let dietary = parts.get(3).copied().unwrap_or("");
        let allergens = parts.get(4).copied().unwrap_or("");
        return t!(
            "intent.ask_menu_item_details.full.reply",
            locale = lang,
            item = name,
            price = price,
            ingredients = ingredients,
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
