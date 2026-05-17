use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::menu_queries::{
    PriceFilter, PriceQuery,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::conversation::application::service::restaurant::{
    ConversationRestaurantMenuService,
};
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskPriceIntentHandler<'a, M> {
    menu_service: &'a ConversationRestaurantMenuService<M>,
}

impl<'a, M> AskPriceIntentHandler<'a, M> {
    pub fn new(menu_service: &'a ConversationRestaurantMenuService<M>) -> Self {
        Self { menu_service }
    }
}

#[async_trait::async_trait]
impl<M> IntentHandler for AskPriceIntentHandler<'_, M>
where
    M: RestaurantMenuRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::AskPrice
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
        let menu_item = self.lookup_entity_value(&input, "menu_item");
        let comparator = self.lookup_entity_value(&input, "price_comparator");
        let amount = self.lookup_entity_value(&input, "price_amount");
        let item = price_item.or(menu_item);
        let raw = self
            .menu_service
            .find_price(
                input.conversation.business_id,
                lang,
                PriceQuery {
                    item: item.map(str::to_string),
                    price_filter: comparator
                        .zip(amount)
                        .map(|(comparator, amount)| PriceFilter {
                            comparator: comparator.to_string(),
                            amount: amount.to_string(),
                        }),
                },
            )
            .await;

        let reply = if let Some(payload) = raw.strip_prefix("price_results:") {
            let parts: Vec<&str> = payload.splitn(3, '|').collect();
            let comp = parts.first().copied().unwrap_or("");
            let amt = parts.get(1).copied().unwrap_or("");
            let items = parts.get(2).copied().unwrap_or("");
            t!(
                "intent.ask_price.results.reply",
                locale = lang,
                comparator = comp,
                amount = amt,
                items = items
            )
            .to_string()
        } else if let Some(payload) = raw.strip_prefix("item_price:") {
            let mut p = payload.splitn(2, '|');
            let name = p.next().unwrap_or("");
            let price = p.next().unwrap_or("");
            t!(
                "intent.ask_price.item_price.reply",
                locale = lang,
                item = name,
                price = price
            )
            .to_string()
        } else if raw.starts_with("no_price_results:") || raw.starts_with("item_not_found:") {
            t!(
                "intent.ask_price.no_results.reply",
                locale = lang,
                item = item.unwrap_or("")
            )
            .to_string()
        } else if let Some(info) = raw.strip_prefix("price_general:") {
            t!("intent.ask_price.general.reply", locale = lang, info = info).to_string()
        } else {
            t!("intent.ask_price.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
