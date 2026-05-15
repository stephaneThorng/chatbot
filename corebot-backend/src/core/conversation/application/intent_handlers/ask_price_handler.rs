use rust_i18n::t;
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::application::port::outbound::restaurant_queries::{
    PriceFilter, PriceQuery,
};
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskPriceIntentHandler<P: RestaurantInformationPort> {
    information_port: Arc<P>,
}

impl<P: RestaurantInformationPort> AskPriceIntentHandler<P> {
    pub fn new(information_port: Arc<P>) -> Self {
        Self { information_port }
    }
}

impl<P: RestaurantInformationPort + Send + Sync> IntentHandler for AskPriceIntentHandler<P> {
    fn intent(&self) -> IntentId {
        IntentId::AskPrice
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let price_item = self.lookup_entity_value(&input, "price_item");
        let menu_item = self.lookup_entity_value(&input, "menu_item");
        let comparator = self.lookup_entity_value(&input, "price_comparator");
        let amount = self.lookup_entity_value(&input, "price_amount");
        let item = price_item.or(menu_item);
        let raw = self.information_port.find_price(PriceQuery {
            item: item.map(str::to_string),
            price_filter: comparator
                .zip(amount)
                .map(|(comparator, amount)| PriceFilter {
                    comparator: comparator.to_string(),
                    amount: amount.to_string(),
                }),
        });

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
