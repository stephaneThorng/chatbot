use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::conversation::application::service::restaurant::{
    business_info_response_formatter::format_opening_hours,
};
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct OpeningHoursIntentHandler<'a, B> {
    business_info_repository: &'a B,
}

impl<'a, B> OpeningHoursIntentHandler<'a, B> {
    pub fn new(business_info_repository: &'a B) -> Self {
        Self {
            business_info_repository,
        }
    }
}

#[async_trait::async_trait]
impl<B> IntentHandler for OpeningHoursIntentHandler<'_, B>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::AskOpeningHours
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let _ = (
            &input.conversation,
            input.analysis_intent,
            input.text,
            input.analysis_entities,
        );
        let reply = self
            .business_info_repository
            .opening_hours(input.conversation.business_id)
            .await
            .map(|hours| format_opening_hours(&hours))
            .unwrap_or_else(|_| "hours_unavailable:".to_string());
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply: vec![reply],
            handled_intent: self.intent(),
        }
    }
}
