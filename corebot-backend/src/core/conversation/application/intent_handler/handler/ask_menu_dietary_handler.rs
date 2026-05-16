use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::menu_queries::MenuDietaryQuery;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_dietary_gateway_port::RestaurantMenuDietaryGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskMenuDietaryIntentHandler<'a, P: RestaurantMenuDietaryGatewayPort + ?Sized> {
    menu_dietary_gateway_port: &'a P,
}

impl<'a, P: RestaurantMenuDietaryGatewayPort + ?Sized> AskMenuDietaryIntentHandler<'a, P> {
    pub fn new(menu_dietary_port: &'a P) -> Self {
        Self {
            menu_dietary_gateway_port: menu_dietary_port,
        }
    }
}

#[async_trait::async_trait]
impl<P: RestaurantMenuDietaryGatewayPort + Send + Sync + ?Sized> IntentHandler
    for AskMenuDietaryIntentHandler<'_, P>
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
            .menu_dietary_gateway_port
            .find_menu_dietary(MenuDietaryQuery {
                dietary_requirement: dietary.map(str::to_string),
            })
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
