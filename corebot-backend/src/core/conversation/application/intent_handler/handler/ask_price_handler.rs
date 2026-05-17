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

        let reply = if let Some(items) = raw.strip_prefix("price_results:") {
            t!(
                "intent.ask_price.results.reply",
                locale = lang,
                comparator = comparator.unwrap_or(""),
                amount = amount.unwrap_or(""),
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
        } else if let Some(payload) = raw.strip_prefix("fallback_full_menu:") {
            t!(
                "intent.ask_price.fallback_full_menu.reply",
                locale = lang,
                items = payload
            )
            .to_string()
        } else if let Some(payload) = raw.strip_prefix("external_menu:") {
            let mut parts = payload.splitn(3, '|');
            let content = parts.next().unwrap_or("");
            let website_url = parts.next().unwrap_or("");
            let pdf_url = parts.next().unwrap_or("");
            let links = format_menu_links(lang, website_url, pdf_url);
            t!(
                "intent.ask_price.external.reply",
                locale = lang,
                content = content,
                links = links
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

fn format_menu_links(lang: &str, website_url: &str, pdf_url: &str) -> String {
    match (website_url.is_empty(), pdf_url.is_empty()) {
        (false, false) => t!(
            "intent.ask_price.external_links.website_and_pdf",
            locale = lang,
            website_url = website_url,
            pdf_url = pdf_url
        )
        .to_string(),
        (false, true) => t!(
            "intent.ask_price.external_links.website_only",
            locale = lang,
            website_url = website_url
        )
        .to_string(),
        (true, false) => t!(
            "intent.ask_price.external_links.pdf_only",
            locale = lang,
            pdf_url = pdf_url
        )
        .to_string(),
        (true, true) => String::new(),
    }
}
