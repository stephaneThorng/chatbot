use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::business_info_queries::EventQuery;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_event_gateway_port::RestaurantEventGatewayPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};
use rust_i18n::t;

pub struct AskEventIntentHandler<'a, P: RestaurantEventGatewayPort + ?Sized> {
    event_gateway_port: &'a P,
}

impl<'a, P: RestaurantEventGatewayPort + ?Sized> AskEventIntentHandler<'a, P> {
    pub fn new(event_port: &'a P) -> Self {
        Self {
            event_gateway_port: event_port,
        }
    }
}

#[async_trait::async_trait]
impl<P: RestaurantEventGatewayPort + Send + Sync + ?Sized> IntentHandler
    for AskEventIntentHandler<'_, P>
{
    fn intent(&self) -> IntentId {
        IntentId::AskEvent
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let location = self.lookup_entity_value(&input, "location");
        let raw = self
            .event_gateway_port
            .find_event_info(EventQuery {
                location: location.map(str::to_string),
            })
            .await;
        let reply = if let Some(payload) = raw.strip_prefix("event_space_available:") {
            let mut p = payload.splitn(2, '|');
            let loc = p.next().unwrap_or("");
            let info = p.next().unwrap_or("");
            t!(
                "intent.ask_event.space_available.reply",
                locale = lang,
                location = loc,
                info = info
            )
            .to_string()
        } else if let Some(payload) = raw.strip_prefix("event_space_unavailable:") {
            let mut p = payload.splitn(2, '|');
            let loc = p.next().unwrap_or("");
            let info = p.next().unwrap_or("");
            t!(
                "intent.ask_event.space_unavailable.reply",
                locale = lang,
                location = loc,
                info = info
            )
            .to_string()
        } else if let Some(info) = raw.strip_prefix("event_info:") {
            t!("intent.ask_event.info.reply", locale = lang, info = info).to_string()
        } else {
            t!("intent.ask_event.reply", locale = lang).to_string()
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
