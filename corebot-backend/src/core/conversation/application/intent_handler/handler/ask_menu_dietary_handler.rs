use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::menu_queries::MenuDietaryQuery;
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
        let dietary = self.lookup_entity_value(&input, "dietary_requirement");
        let raw = self
            .menu_service
            .find_menu_dietary(
                input.conversation.business_id,
                lang,
                MenuDietaryQuery {
                    dietary_requirement: dietary.map(str::to_string),
                },
            )
            .await;
        let reply = parse_dietary_reply(&raw, lang, dietary);
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}

fn parse_dietary_reply(raw: &str, lang: &str, _dietary: Option<&str>) -> String {
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
