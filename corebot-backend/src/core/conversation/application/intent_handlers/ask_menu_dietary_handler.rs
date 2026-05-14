use rust_i18n::t;
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::application::port::outbound::restaurant_queries::MenuDietaryQuery;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskMenuDietaryIntentHandler<P: RestaurantInformationPort> {
    information_port: Arc<P>,
}

impl<P: RestaurantInformationPort> AskMenuDietaryIntentHandler<P> {
    pub fn new(information_port: Arc<P>) -> Self {
        Self { information_port }
    }
}

impl<P: RestaurantInformationPort + Send + Sync> IntentHandler for AskMenuDietaryIntentHandler<P> {
    fn intent(&self) -> IntentId {
        IntentId::AskMenuDietary
    }

    fn config(&self) -> IntentConfig {
        IntentConfig { id: self.intent(), workflow: IntentWorkflow::Informational }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let dietary = self.lookup_entity_value(&input, "dietary_requirement");
        let raw = self.information_port.find_menu_dietary(MenuDietaryQuery {
            dietary_requirement: dietary.map(str::to_string),
        });
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
        return t!(
            "intent.ask_menu_dietary.no_results.reply",
            locale = lang,
            requirement = req
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
