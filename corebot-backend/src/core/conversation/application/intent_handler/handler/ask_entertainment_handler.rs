use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskEntertainmentIntentHandler<'a, B> {
    business_info_repository: &'a B,
}

impl<'a, B> AskEntertainmentIntentHandler<'a, B> {
    pub fn new(business_info_repository: &'a B) -> Self {
        Self {
            business_info_repository,
        }
    }
}

#[async_trait::async_trait]
impl<B> IntentHandler for AskEntertainmentIntentHandler<'_, B>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::AskEntertainment
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
            .facts(input.conversation.business_id, lang)
            .await
        {
            Ok(facts) => facts
                .iter()
                .find(|fact| fact.fact_type == "entertainment")
                .map(|fact| {
                    t!(
                        "intent.ask_entertainment.confirmed.reply",
                        locale = lang,
                        info = fact.content.as_str()
                    )
                    .to_string()
                })
                .unwrap_or_else(|| t!("intent.ask_entertainment.reply", locale = lang).to_string()),
            Err(_) => t!("intent.ask_entertainment.reply", locale = lang).to_string(),
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
