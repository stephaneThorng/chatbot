use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::conversation::application::service::restaurant::{
    business_info_response_formatter::facility_matches,
};
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};
use rust_i18n::t;

pub struct AskFacilitiesIntentHandler<'a, B> {
    business_info_repository: &'a B,
}

impl<'a, B> AskFacilitiesIntentHandler<'a, B> {
    pub fn new(business_info_repository: &'a B) -> Self {
        Self {
            business_info_repository,
        }
    }
}

#[async_trait::async_trait]
impl<B> IntentHandler for AskFacilitiesIntentHandler<'_, B>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::AskFacilities
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let facility = self.lookup_entity_value(&input, "facility");
        let reply = match self
            .business_info_repository
            .facilities(input.conversation.business_id)
            .await
        {
            Ok(facilities) => {
                if let Some(facility) = facility {
                    if facilities
                        .iter()
                        .any(|candidate| facility_matches(&candidate.label, facility))
                    {
                        t!(
                            "intent.ask_facilities.available.reply",
                            locale = lang,
                            facility = facility
                        )
                        .to_string()
                    } else {
                        t!(
                            "intent.ask_facilities.unavailable.reply",
                            locale = lang,
                            facility = facility
                        )
                        .to_string()
                    }
                } else {
                    t!(
                        "intent.ask_facilities.all.reply",
                        locale = lang,
                        facilities = facilities
                            .iter()
                            .map(|facility| facility.label.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                    .to_string()
                }
            }
            Err(_) => t!("intent.ask_facilities.reply", locale = lang).to_string(),
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
