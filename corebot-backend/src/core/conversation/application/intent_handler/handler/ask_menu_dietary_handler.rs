use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::menu_queries::{
    MenuDietaryQuery, MenuQuery,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::conversation::application::service::restaurant::{
    ConversationRestaurantMenuService,
};
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskMenuDietaryIntentHandler<'a, M> {
    menu_service: &'a ConversationRestaurantMenuService<M>,
}

impl<'a, M> AskMenuDietaryIntentHandler<'a, M> {
    pub fn new(menu_service: &'a ConversationRestaurantMenuService<M>) -> Self {
        Self { menu_service }
    }
}

#[async_trait::async_trait]
impl<M> IntentHandler for AskMenuDietaryIntentHandler<'_, M>
where
    M: RestaurantMenuRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::AskMenuDietary
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let dietary = input
            .analysis_entities
            .iter()
            .filter(|entity| entity.entity_label == "dietary_requirement")
            .map(|entity| entity.value.clone())
            .collect::<Vec<_>>();
        let menu_item = self.lookup_entity_value(&input, "menu_item");
        let reply = if dietary.is_empty() {
            if let Some(menu_item) = menu_item {
                let raw = self
                    .menu_service
                    .find_menu(
                        input.conversation.business_id,
                        lang,
                        MenuQuery {
                            price_item: Some(menu_item.to_string()),
                            price_filter: None,
                            cheapest_only: false,
                            exclude_item: mentions_exclusion(input.text, menu_item),
                        },
                    )
                    .await;
                parse_menu_fallback_reply(&raw, lang)
            } else {
                t!("intent.ask_menu_dietary.reply", locale = lang).to_string()
            }
        } else if dietary.iter().all(|value| !looks_like_dietary_tag(value)) {
            let raw = self
                .menu_service
                .find_menu(
                    input.conversation.business_id,
                    lang,
                    MenuQuery {
                        price_item: Some(dietary[0].clone()),
                        price_filter: None,
                        cheapest_only: false,
                        exclude_item: false,
                    },
                )
                .await;
            parse_menu_fallback_reply(&raw, lang)
        } else {
            let raw = self
                .menu_service
                .find_menu_dietary(
                    input.conversation.business_id,
                    lang,
                    MenuDietaryQuery {
                        dietary_requirements: dietary.clone(),
                    },
                )
                .await;
            parse_dietary_reply(&raw, lang)
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply: vec![reply],
            handled_intent: self.intent(),
        }
    }
}

fn parse_dietary_reply(raw: &str, lang: &str) -> String {
    if let Some(payload) = raw.strip_prefix("dietary_results:") {
        let mut parts = payload.splitn(2, '|');
        let req = parts.next().unwrap_or("");
        let items = parts.next().unwrap_or("");
        return t!(
            "intent.ask_menu_dietary.results.reply",
            locale = lang,
            requirement = req,
            items = items
        )
        .to_string();
    }
    if let Some(req) = raw.strip_prefix("no_dietary:") {
        let mut parts = req.splitn(2, '|');
        let requirement = parts.next().unwrap_or("");
        let options = parts.next().unwrap_or("");
        return t!(
            "intent.ask_menu_dietary.no_results.reply",
            locale = lang,
            requirement = requirement,
            options = options
        )
        .to_string();
    }
    if let Some(options) = raw.strip_prefix("dietary_no_filter:") {
        return t!(
            "intent.ask_menu_dietary.options.reply",
            locale = lang,
            options = options
        )
        .to_string();
    }
    t!("intent.ask_menu_dietary.reply", locale = lang).to_string()
}

fn parse_menu_fallback_reply(raw: &str, lang: &str) -> String {
    if let Some(payload) = raw
        .strip_prefix("ingredient_results:")
        .or_else(|| raw.strip_prefix("ingredient_exclusion_results:"))
    {
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
        return t!(
            "intent.ask_menu_general.external.reply",
            locale = lang,
            content = content,
            links = format_links(website_url, pdf_url)
        )
        .to_string();
    }
    t!("intent.ask_menu_dietary.reply", locale = lang).to_string()
}

fn looks_like_dietary_tag(value: &str) -> bool {
    matches!(
        value.to_lowercase().replace('-', " ").as_str(),
        "vegan" | "vegetarian" | "halal" | "gluten free" | "dairy free" | "nut free" | "nuts free"
    )
}

fn mentions_exclusion(text: &str, item: &str) -> bool {
    let normalized_text = text.to_lowercase();
    let normalized_item = item.to_lowercase();
    normalized_text.contains(&format!("without {normalized_item}"))
        || normalized_text.contains(&format!("no {normalized_item}"))
}

fn format_links(website_url: &str, pdf_url: &str) -> String {
    match (website_url.is_empty(), pdf_url.is_empty()) {
        (false, false) => format!("Website: {website_url}. PDF: {pdf_url}."),
        (false, true) => format!("Website: {website_url}."),
        (true, false) => format!("PDF: {pdf_url}."),
        (true, true) => String::new(),
    }
}
