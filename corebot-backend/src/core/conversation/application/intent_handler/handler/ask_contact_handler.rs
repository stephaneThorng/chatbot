use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskContactIntentHandler<'a, B> {
    business_info_repository: &'a B,
}

impl<'a, B> AskContactIntentHandler<'a, B> {
    pub fn new(business_info_repository: &'a B) -> Self {
        Self {
            business_info_repository,
        }
    }
}

#[async_trait::async_trait]
impl<B> IntentHandler for AskContactIntentHandler<'_, B>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::AskContact
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let reply = match self
            .business_info_repository
            .contact_channels(input.conversation.business_id)
            .await
        {
            Ok(channels) => {
                let phone = channels
                    .iter()
                    .find(|channel| channel.channel_type == "phone")
                    .map(|channel| channel.value.as_str())
                    .unwrap_or("");
                let email = channels
                    .iter()
                    .find(|channel| channel.channel_type == "email")
                    .map(|channel| channel.value.as_str())
                    .unwrap_or("");
                t!(
                    "intent.ask_contact.data.reply",
                    locale = lang,
                    phone = phone,
                    email = email
                )
                .to_string()
            }
            Err(_) => t!("intent.ask_contact.reply", locale = lang).to_string(),
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
