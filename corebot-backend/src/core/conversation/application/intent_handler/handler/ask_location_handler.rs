use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskLocationIntentHandler<'a, B> {
    business_info_repository: &'a B,
}

impl<'a, B> AskLocationIntentHandler<'a, B> {
    pub fn new(business_info_repository: &'a B) -> Self {
        Self {
            business_info_repository,
        }
    }
}

#[async_trait::async_trait]
impl<B> IntentHandler for AskLocationIntentHandler<'_, B>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::AskLocation
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let near = self.lookup_entity_value(&input, "location");
        let reply = match self
            .business_info_repository
            .location(input.conversation.business_id)
            .await
        {
            Ok(Some(location)) => {
                let address = match location.nearby_description {
                    Some(nearby) if !nearby.is_empty() => {
                        format!("{} - {}", location.address_line, nearby)
                    }
                    _ => location.address_line,
                };
                if let Some(near) = near {
                    if address.to_lowercase().contains(&near.to_lowercase()) {
                        t!(
                            "intent.ask_location.near_confirmed.reply",
                            locale = lang,
                            location = near,
                            address = address
                        )
                        .to_string()
                    } else {
                        t!(
                            "intent.ask_location.near_denied.reply",
                            locale = lang,
                            location = near,
                            address = address
                        )
                        .to_string()
                    }
                } else {
                    t!(
                        "intent.ask_location.address.reply",
                        locale = lang,
                        address = address
                    )
                    .to_string()
                }
            }
            _ => t!("intent.ask_location.reply", locale = lang).to_string(),
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
