use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::menu_queries::{
    MenuQuery, PriceFilter,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::conversation::application::service::restaurant::{
    ConversationRestaurantMenuService,
};
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskMenuGeneralIntentHandler<'a, M> {
    menu_service: &'a ConversationRestaurantMenuService<M>,
}

impl<'a, M> AskMenuGeneralIntentHandler<'a, M> {
    pub fn new(menu_service: &'a ConversationRestaurantMenuService<M>) -> Self {
        Self { menu_service }
    }
}

#[async_trait::async_trait]
impl<M> IntentHandler for AskMenuGeneralIntentHandler<'_, M>
where
    M: RestaurantMenuRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::AskMenuGeneral
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let price_item = self.lookup_entity_value(&input, "price_item");
        let comparator = self.lookup_entity_value(&input, "price_comparator");
        let amount = self.lookup_entity_value(&input, "price_amount");

        let raw = self
            .menu_service
            .find_menu(
                input.conversation.business_id,
                lang,
                MenuQuery {
                    price_item: price_item.map(str::to_string),
                    price_filter: comparator
                        .zip(amount)
                        .map(|(comparator, amount)| PriceFilter {
                            comparator: comparator.to_string(),
                            amount: amount.to_string(),
                        }),
                },
            )
            .await;
        let reply = parse_menu_reply(&raw, lang, comparator, amount, price_item);

        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}

fn parse_menu_reply(
    raw: &str,
    lang: &str,
    comparator: Option<&str>,
    amount: Option<&str>,
    price_item: Option<&str>,
) -> String {
    if let Some(payload) = raw.strip_prefix("price_results:") {
        return t!(
            "intent.ask_menu_general.price_results.reply",
            locale = lang,
            comparator = comparator.unwrap_or(""),
            amount = amount.unwrap_or(""),
            items = payload
        )
        .to_string();
    }
    if raw.starts_with("no_results:") {
        return t!(
            "intent.ask_menu_general.no_results.reply",
            locale = lang,
            comparator = comparator.unwrap_or(""),
            amount = amount.unwrap_or("")
        )
        .to_string();
    }
    if let Some(payload) = raw.strip_prefix("item_found:") {
        let parts: Vec<&str> = payload.splitn(3, '|').collect();
        let name = parts.first().copied().unwrap_or("");
        let price = parts.get(1).copied().unwrap_or("");
        return t!(
            "intent.ask_menu_general.item_found.reply",
            locale = lang,
            item = name,
            price = price
        )
        .to_string();
    }
    if raw.starts_with("item_not_found:") {
        return t!(
            "intent.ask_menu_general.item_not_found.reply",
            locale = lang,
            item = price_item.unwrap_or("")
        )
        .to_string();
    }
    if let Some(payload) = raw.strip_prefix("full_menu:") {
        return t!(
            "intent.ask_menu_general.full_menu.reply",
            locale = lang,
            items = payload
        )
        .to_string();
    }
    t!("intent.ask_menu_general.reply", locale = lang).to_string()
}
