use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::menu_queries::MenuQuery;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::conversation::application::service::restaurant::{
    ConversationRestaurantMenuService,
};
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};
use crate::core::conversation::domain::restaurant::model::AmountComparator;

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
        let menu_item = self.lookup_entity_value(&input, "menu_item");
        let comparator = self.lookup_entity_value(&input, "price_comparator");
        let amount = self.lookup_entity_value(&input, "price_amount");
        let item = price_item.or(menu_item);
        let cheapest_only = mentions_cheapest(input.text);

        let raw = self
            .menu_service
            .find_menu(
                input.conversation.business_id,
                lang,
                MenuQuery {
                    price_item: item.map(str::to_string),
                    price_filter: comparator.zip(amount).and_then(|(comparator, amount)| {
                        parse_amount_comparator(comparator, amount)
                    }),
                    cheapest_only,
                    exclude_item: item.is_some_and(|value| mentions_exclusion(input.text, value)),
                },
            )
            .await;
        let reply = parse_menu_reply(&raw, lang, comparator, amount, item);

        StateHandlerResult {
            updated_conversation: input.conversation,
            reply: vec![reply],
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
    if let Some(payload) = raw.strip_prefix("ingredient_results:") {
        return t!(
            "intent.ask_menu_general.full_menu.reply",
            locale = lang,
            items = payload
        )
        .to_string();
    }
    if let Some(payload) = raw.strip_prefix("ingredient_exclusion_results:") {
        return t!(
            "intent.ask_menu_general.full_menu.reply",
            locale = lang,
            items = payload
        )
        .to_string();
    }
    if let Some(payload) = raw.strip_prefix("fallback_full_menu:") {
        return t!(
            "intent.ask_menu_general.fallback_full_menu.reply",
            locale = lang,
            items = payload
        )
        .to_string();
    }
    if let Some(payload) = raw.strip_prefix("external_menu:") {
        let mut parts = payload.splitn(3, '|');
        let content = parts.next().unwrap_or("");
        let website_url = parts.next().unwrap_or("");
        let pdf_url = parts.next().unwrap_or("");
        let links = format_menu_links(lang, website_url, pdf_url);
        let content_prefix = if content.is_empty() {
            String::new()
        } else {
            format!("{content} ")
        };
        return t!(
            "intent.ask_menu_general.external.reply",
            locale = lang,
            content = content_prefix.trim_end(),
            links = links
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

fn format_menu_links(lang: &str, website_url: &str, pdf_url: &str) -> String {
    match (website_url.is_empty(), pdf_url.is_empty()) {
        (false, false) => t!(
            "intent.ask_menu_general.external_links.website_and_pdf",
            locale = lang,
            website_url = website_url,
            pdf_url = pdf_url
        )
        .to_string(),
        (false, true) => t!(
            "intent.ask_menu_general.external_links.website_only",
            locale = lang,
            website_url = website_url
        )
        .to_string(),
        (true, false) => t!(
            "intent.ask_menu_general.external_links.pdf_only",
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
