use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::menu_queries::PriceQuery;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::conversation::application::service::restaurant::{
    ConversationRestaurantMenuService,
};
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};
use crate::core::conversation::domain::restaurant::model::AmountComparator;

pub struct AskPriceIntentHandler<'a, M, B> {
    menu_service: &'a ConversationRestaurantMenuService<M>,
    business_info_repository: &'a B,
}

impl<'a, M, B> AskPriceIntentHandler<'a, M, B> {
    pub fn new(
        menu_service: &'a ConversationRestaurantMenuService<M>,
        business_info_repository: &'a B,
    ) -> Self {
        Self {
            menu_service,
            business_info_repository,
        }
    }
}

#[async_trait::async_trait]
impl<M, B> IntentHandler for AskPriceIntentHandler<'_, M, B>
where
    M: RestaurantMenuRepositoryPort + Send + Sync,
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
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
        if mentions_taxes(input.text) {
            let reply = match self
                .business_info_repository
                .facts(input.conversation.business_id, lang)
                .await
            {
                Ok(facts) => facts
                    .iter()
                    .find(|fact| fact.fact_type == "taxes")
                    .map(|fact| {
                        t!(
                            "intent.ask_price.taxes.reply",
                            locale = lang,
                            info = fact.content.as_str()
                        )
                        .to_string()
                    })
                    .unwrap_or_else(|| t!("intent.ask_price.reply", locale = lang).to_string()),
                Err(_) => t!("intent.ask_price.reply", locale = lang).to_string(),
            };
            return StateHandlerResult {
                updated_conversation: input.conversation,
                reply: vec![reply],
                handled_intent: self.intent(),
            };
        }

        let price_item = self.lookup_entity_value(&input, "price_item");
        let menu_item = self.lookup_entity_value(&input, "menu_item");
        let comparator = self.lookup_entity_value(&input, "price_comparator");
        let amount = self.lookup_entity_value(&input, "price_amount");
        let item = price_item.or(menu_item);
        let cheapest_only = mentions_cheapest(input.text);
        let raw = self
            .menu_service
            .find_price(
                input.conversation.business_id,
                lang,
                PriceQuery {
                    item: item.map(str::to_string),
                    price_filter: comparator.zip(amount).and_then(|(comparator, amount)| {
                        parse_amount_comparator(comparator, amount)
                    }),
                    cheapest_only,
                    exclude_item: item.is_some_and(|value| mentions_exclusion(input.text, value)),
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
            reply: vec![reply],
            handled_intent: self.intent(),
        }
    }
}

fn mentions_cheapest(text: &str) -> bool {
    let normalized = text.to_lowercase();
    normalized.contains("cheapest") || normalized.contains("lowest price")
}

fn mentions_exclusion(text: &str, item: &str) -> bool {
    let normalized_text = text.to_lowercase();
    let normalized_item = item.to_lowercase();
    normalized_text.contains(&format!("without {normalized_item}"))
        || normalized_text.contains(&format!("no {normalized_item}"))
}

fn mentions_taxes(text: &str) -> bool {
    let normalized = text.to_lowercase();
    normalized.contains("tax") || normalized.contains("taxes") || normalized.contains("service charge")
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

fn parse_amount_comparator(comparator: &str, amount: &str) -> Option<AmountComparator> {
    let normalized = comparator.trim().to_lowercase();
    let amount = parse_amount(amount)?;

    match normalized.as_str() {
        "under" | "less than" | "below" => Some(AmountComparator::Under(amount)),
        "greater than" | "more than" | "over" | "above" => Some(AmountComparator::Above(amount)),
        "at least" | "minimum" | "min" | "from" => Some(AmountComparator::AtLeast(amount)),
        "at most" | "maximum" | "max" | "up to" => Some(AmountComparator::AtMost(amount)),
        "equal" | "equals" | "exactly" => Some(AmountComparator::Equal(amount)),
        _ => None,
    }
}

fn parse_amount(amount: &str) -> Option<i32> {
    let normalized = amount.trim().to_lowercase();
    let compact = normalized
        .replace("euros", "")
        .replace("euro", "")
        .replace("eur", "")
        .replace("dollars", "")
        .replace("dollar", "")
        .replace("usd", "")
        .replace("idr", "")
        .replace('$', "")
        .trim()
        .to_string();

    if let Some(value) = compact.strip_suffix('k') {
        return value.trim().parse::<i32>().ok().map(|number| number * 1_000);
    }

    compact.parse::<i32>().ok()
}
