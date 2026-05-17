use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};
use rust_i18n::t;

pub struct AskEventIntentHandler<'a, B> {
    business_info_repository: &'a B,
}

impl<'a, B> AskEventIntentHandler<'a, B> {
    pub fn new(business_info_repository: &'a B) -> Self {
        Self {
            business_info_repository,
        }
    }
}

#[async_trait::async_trait]
impl<B> IntentHandler for AskEventIntentHandler<'_, B>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
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
        let reply = match self
            .business_info_repository
            .event_spaces(input.conversation.business_id)
            .await
        {
            Ok(spaces) => {
                if let Some(location) = location {
                    if let Some(space) = spaces
                        .iter()
                        .find(|space| location.to_lowercase().contains(&space.name.to_lowercase()))
                    {
                        let info = space
                            .contact
                            .clone()
                            .or_else(|| space.description.clone())
                            .unwrap_or_default();
                        t!(
                            "intent.ask_event.space_available.reply",
                            locale = lang,
                            location = location,
                            info = info
                        )
                        .to_string()
                    } else {
                        let info = format!(
                            "We have {} available for events.",
                            spaces
                                .iter()
                                .map(|space| space.name.clone())
                                .collect::<Vec<_>>()
                                .join(" and ")
                        );
                        t!(
                            "intent.ask_event.space_unavailable.reply",
                            locale = lang,
                            location = location,
                            info = info
                        )
                        .to_string()
                    }
                } else {
                    let info = spaces
                        .iter()
                        .map(|space| {
                            let description = space.description.clone().unwrap_or_default();
                            format!("{} {}", space.name, description).trim().to_string()
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    t!("intent.ask_event.info.reply", locale = lang, info = info).to_string()
                }
            }
            Err(_) => t!("intent.ask_event.reply", locale = lang).to_string(),
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply: vec![reply],
            handled_intent: self.intent(),
        }
    }
}
