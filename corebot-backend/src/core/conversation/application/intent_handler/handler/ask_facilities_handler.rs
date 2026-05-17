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
        let facility = self
            .lookup_entity_value(&input, "facility")
            .map(str::to_string);
        let reply = match self
            .business_info_repository
            .facilities(input.conversation.business_id)
            .await
        {
            Ok(facilities) => {
                let requested = facility.or_else(|| infer_facility_from_text(input.text, &facilities));
                if let Some(facility) = requested.as_deref() {
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
                        facilities = format_bullet_list(
                            &facilities
                                .iter()
                                .map(|facility| facility.label.clone())
                                .collect::<Vec<_>>()
                        )
                    )
                    .to_string()
                }
            }
            Err(_) => t!("intent.ask_facilities.reply", locale = lang).to_string(),
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply: vec![reply],
            handled_intent: self.intent(),
        }
    }
}

fn infer_facility_from_text(
    text: &str,
    facilities: &[crate::core::conversation::domain::restaurant::model::Facility],
) -> Option<String> {
    let normalized = text.to_lowercase();
    if normalized.contains("parking") {
        return Some("parking".to_string());
    }
    if normalized.contains(" air conditioning")
        || normalized.starts_with("ac ")
        || normalized.ends_with(" ac")
        || normalized.contains(" ac ")
    {
        return Some("ac".to_string());
    }

    facilities
        .iter()
        .find(|facility| normalized.contains(&facility.label.to_lowercase()))
        .map(|facility| facility.label.clone())
}

fn format_bullet_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("- {value}"))
        .collect::<Vec<_>>()
        .join("\n")
}
