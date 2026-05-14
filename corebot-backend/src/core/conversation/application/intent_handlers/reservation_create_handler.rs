use chrono::{Datelike, NaiveDate, Weekday};
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult, WorkflowPostProcessResult,
};
use crate::core::conversation::application::port::outbound::restaurant_queries::{
    ReservationCreateQuery, ReservationFailure,
};
use crate::core::conversation::application::port::outbound::restaurant_reservation_port::RestaurantReservationPort;
use crate::core::conversation::domain::date_resolver::{resolve_date, resolve_time};
use crate::core::conversation::domain::model::conversation::Conversation;
use crate::core::conversation::domain::model::intent::{
    IntentConfig, IntentId, IntentWorkflow, NluTask, WorkflowConfig, i18n_key,
};
use crate::core::conversation::domain::model::slot::{
    SlotConfig, SlotConstraint, SlotConstraintEntry, SlotDataValue, SlotName,
};

pub struct ReservationCreateIntentHandler<P: RestaurantReservationPort + ?Sized> {
    reservation_port: Arc<P>,
}

impl<P: RestaurantReservationPort + ?Sized> ReservationCreateIntentHandler<P> {
    pub fn new(reservation_port: Arc<P>) -> Self {
        Self { reservation_port }
    }

    fn workflow_value<'a>(conversation: &'a Conversation, slot: SlotName) -> Option<&'a SlotDataValue> {
        conversation
            .active_workflow()
            .and_then(|workflow| workflow.slot_value(slot))
    }

    fn workflow_text(conversation: &Conversation, slot: SlotName) -> String {
        match Self::workflow_value(conversation, slot) {
            Some(SlotDataValue::Text(value))
            | Some(SlotDataValue::Date(value))
            | Some(SlotDataValue::Time(value)) => value.clone(),
            Some(SlotDataValue::Number(value)) => value.to_string(),
            Some(SlotDataValue::Boolean(value)) => value.to_string(),
            None => String::new(),
        }
    }

    fn workflow_people_count(conversation: &Conversation) -> u32 {
        match Self::workflow_value(conversation, SlotName::People) {
            Some(SlotDataValue::Number(value)) => *value,
            _ => 0,
        }
    }

    fn resolved_date_text(conversation: &Conversation) -> String {
        let raw_date = Self::workflow_text(conversation, SlotName::Date);
        if raw_date.is_empty() {
            return raw_date;
        }
        match resolve_date(&raw_date) {
            Ok(date) => format_resolved_date(date),
            Err(_) => raw_date,
        }
    }

    fn confirmation_summary(conversation: &Conversation) -> String {
        rust_i18n::t!(
            "workflow.reservation_create.confirmation.prompt",
            locale = conversation.lang.as_str(),
            name = Self::workflow_text(conversation, SlotName::Name),
            date = Self::resolved_date_text(conversation),
            time = Self::workflow_text(conversation, SlotName::Time),
            people = Self::workflow_people_count(conversation)
        )
        .to_string()
    }
}

impl<P: RestaurantReservationPort + Send + Sync + ?Sized> IntentHandler
    for ReservationCreateIntentHandler<P>
{
    fn intent(&self) -> IntentId {
        IntentId::ReservationCreate
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Workflow(WorkflowConfig {
                nlu_task: Some(NluTask::ReservationCreate),
                slots: vec![
                    SlotConfig {
                        name: SlotName::Name,
                        required: true,
                        prompt: i18n_key("workflow.reservation_create.slot.name.prompt"),
                        constraints: vec![SlotConstraintEntry::new(SlotConstraint::TextMaxLen(100))],
                    },
                    SlotConfig {
                        name: SlotName::Date,
                        required: true,
                        prompt: i18n_key("workflow.reservation_create.slot.date.prompt"),
                        constraints: vec![SlotConstraintEntry::with_error_key(
                            SlotConstraint::FutureDate,
                            "workflow.reservation_create.past_date.error",
                        )],
                    },
                    SlotConfig {
                        name: SlotName::Time,
                        required: true,
                        prompt: i18n_key("workflow.reservation_create.slot.time.prompt"),
                        constraints: vec![],
                    },
                    SlotConfig {
                        name: SlotName::People,
                        required: true,
                        prompt: i18n_key("workflow.reservation_create.slot.people.prompt"),
                        constraints: vec![SlotConstraintEntry::new(SlotConstraint::NumberRange(1, 20))],
                    },
                ],
                starting_message: Some(i18n_key("workflow.reservation_create.starting.message")),
                confirmation_prompt: Some(i18n_key("workflow.reservation_create.confirmation.prompt")),
                completion_response: Some(i18n_key("workflow.reservation_create.completion.success")),
            }),
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        self.handle_workflow(input)
    }

    fn negative_prompt(&self, lang: &str) -> String {
        rust_i18n::t!("workflow.reservation_create.update.prompt", locale = lang).to_string()
    }

    fn confirmation_prompt(
        &self,
        _workflow_cfg: &crate::core::conversation::domain::model::intent::WorkflowConfig,
        conversation: &Conversation,
    ) -> String {
        Self::confirmation_summary(conversation)
    }

    fn post_process(
        &self,
        lang: &str,
        mut conversation: Conversation,
    ) -> WorkflowPostProcessResult {
        let name = Self::workflow_text(&conversation, SlotName::Name);
        let raw_date = Self::workflow_text(&conversation, SlotName::Date);
        let raw_time = Self::workflow_text(&conversation, SlotName::Time);
        let people_count = Self::workflow_people_count(&conversation);

        let Ok(date) = resolve_date(&raw_date) else {
            conversation.clear_workflow_slot(SlotName::Date);
            conversation.clear_workflow_slot(SlotName::Time);
            return WorkflowPostProcessResult::Failed {
                updated_conversation: conversation,
                reply: rust_i18n::t!(
                    "workflow.reservation_create.slot.date.prompt",
                    locale = lang
                )
                .to_string(),
            };
        };

        let Ok(time) = resolve_time(&raw_time) else {
            conversation.clear_workflow_slot(SlotName::Time);
            return WorkflowPostProcessResult::Failed {
                updated_conversation: conversation,
                reply: rust_i18n::t!(
                    "workflow.reservation_create.time_invalid.error",
                    locale = lang
                )
                .to_string(),
            };
        };

        let date_display = format_resolved_date(date);
        let time_display = raw_time.clone();

        match self.reservation_port.create_reservation(ReservationCreateQuery {
            name: name.clone(),
            date,
            time,
            people_count,
        }) {
            Ok(creation) => {
                let reference = creation
                    .strip_prefix("created:")
                    .unwrap_or(creation.as_str())
                    .to_string();
                conversation.remember_customer_name(name.clone());
                conversation.remember_reservation_reference(reference.clone());
                WorkflowPostProcessResult::Succeeded {
                    updated_conversation: conversation,
                    reply: Some(
                        rust_i18n::t!(
                            "workflow.reservation_create.completion.success",
                            locale = lang,
                            name = name,
                            date = date_display,
                            time = time_display,
                            people = people_count,
                            reference = reference
                        )
                        .to_string(),
                    ),
                }
            }
            Err(ReservationFailure::RestaurantClosed) => {
                conversation.clear_workflow_slot(SlotName::Date);
                conversation.clear_workflow_slot(SlotName::Time);
                WorkflowPostProcessResult::Failed {
                    updated_conversation: conversation,
                    reply: rust_i18n::t!(
                        "workflow.reservation_create.closed.error",
                        locale = lang
                    )
                    .to_string(),
                }
            }
            Err(ReservationFailure::NoAvailability { next_slot }) => {
                conversation.clear_workflow_slot(SlotName::Date);
                conversation.clear_workflow_slot(SlotName::Time);
                let reply = match next_slot {
                    Some(suggestion) => rust_i18n::t!(
                        "workflow.reservation_create.no_availability_with_suggestion.error",
                        locale = lang,
                        next_slot = suggestion
                    )
                    .to_string(),
                    None => rust_i18n::t!(
                        "workflow.reservation_create.no_availability.error",
                        locale = lang
                    )
                    .to_string(),
                };
                WorkflowPostProcessResult::Failed {
                    updated_conversation: conversation,
                    reply,
                }
            }
        }
    }
}

fn format_resolved_date(date: NaiveDate) -> String {
    format!(
        "{} {} {} {}",
        weekday_name(date.weekday()),
        month_name(date.month()),
        date.day(),
        date.year()
    )
}

fn weekday_name(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::conversation::application::port::outbound::restaurant_queries::ReservationFailure;
    use crate::core::conversation::domain::conversation::Conversation;
    use crate::core::conversation::domain::domain_type::DomainType;
    use crate::core::conversation::domain::model::slot::{SlotDataValue, SlotName};
    use crate::core::conversation::application::nlu_analysis_result::NluEntityResult;
    use std::sync::Arc;

    struct StubReservationPort;
    impl RestaurantReservationPort for StubReservationPort {
        fn create_reservation(&self, _: ReservationCreateQuery) -> Result<String, ReservationFailure> {
            Ok("created:REST-NEW123".to_string())
        }
        fn check_reservation(
            &self,
            _: crate::core::conversation::application::port::outbound::restaurant_queries::ReservationLookupQuery,
        ) -> String {
            "no_reference_or_name:".to_string()
        }
    }

    struct FullStubReservationPort;
    impl RestaurantReservationPort for FullStubReservationPort {
        fn create_reservation(&self, _: ReservationCreateQuery) -> Result<String, ReservationFailure> {
            Err(ReservationFailure::NoAvailability {
                next_slot: Some("Monday June 1 at 21:00".to_string()),
            })
        }
        fn check_reservation(
            &self,
            _: crate::core::conversation::application::port::outbound::restaurant_queries::ReservationLookupQuery,
        ) -> String {
            "no_reference_or_name:".to_string()
        }
    }

    struct ClosedStubReservationPort;
    impl RestaurantReservationPort for ClosedStubReservationPort {
        fn create_reservation(&self, _: ReservationCreateQuery) -> Result<String, ReservationFailure> {
            Err(ReservationFailure::RestaurantClosed)
        }
        fn check_reservation(
            &self,
            _: crate::core::conversation::application::port::outbound::restaurant_queries::ReservationLookupQuery,
        ) -> String {
            "no_reference_or_name:".to_string()
        }
    }

    fn entity(entity_label: &'static str, value: &str) -> NluEntityResult {
        NluEntityResult {
            entity_label: entity_label.to_string(),
            value: value.to_string(),
            raw_value: value.to_string(),
            start: 0,
            end: value.len(),
            confidence: 1.0,
        }
    }

    fn handler() -> ReservationCreateIntentHandler<StubReservationPort> {
        ReservationCreateIntentHandler::new(Arc::new(StubReservationPort))
    }

    fn handle(
        conversation: Conversation,
        intent: IntentId,
        text: &str,
        entities: Vec<NluEntityResult>,
    ) -> StateHandlerResult {
        handler().handle(IntentHandlerInput {
            conversation,
            analysis_intent: &intent,
            text,
            analysis_entities: &entities,
        })
    }

    #[test]
    fn idle_workflow_prompts_for_first_missing_slot() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let result = handle(conversation, IntentId::ReservationCreate, "", vec![]);
        assert!(result.reply.ends_with("What name should I use for the reservation?"));
        assert!(result.updated_conversation.has_active_workflow());
    }

    #[test]
    fn missing_slots_are_filled_from_entities() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let result = handle(
            conversation,
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
            ],
        );
        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(workflow.slot_value(SlotName::Name), Some(&SlotDataValue::Text("Alice".to_string())));
        assert!(result.reply.ends_with("For how many people?"));
    }

    #[test]
    fn filled_workflow_asks_for_confirmation() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let result = handle(
            conversation,
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4 people"),
            ],
        );
        assert!(result.reply.contains("Alice"));
        assert!(result.reply.contains("7pm"));
        assert!(result.reply.contains("4 people"));
        assert!(result.reply.contains("Do you confirm"));
    }

    #[test]
    fn bare_numeric_reply_fills_people_slot_when_it_is_next() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
            ],
        )
        .updated_conversation;
        let result = handler().handle(IntentHandlerInput {
            conversation,
            analysis_intent: &IntentId::ReservationCreate,
            text: "10",
            analysis_entities: &[],
        });
        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(workflow.slot_value(SlotName::People), Some(&SlotDataValue::Number(10)));
        assert!(result.reply.contains("10 people"));
        assert!(result.reply.contains("Do you confirm"));
    }

    #[test]
    fn negative_confirmation_keeps_workflow_open_for_changes() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4 people"),
            ],
        )
        .updated_conversation;
        let result = handle(conversation, IntentId::Negative, "", vec![]);
        assert_eq!(result.reply, "Okay. What would you like to change?");
        assert!(result.updated_conversation.has_active_workflow());
    }

    #[test]
    fn updated_slot_reasks_for_confirmation() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4 people"),
            ],
        )
        .updated_conversation;
        let result = handle(conversation, IntentId::Negative, "", vec![entity("people_count", "5 people")]);
        assert!(result.reply.contains("Do you confirm"));
        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(workflow.slot_value(SlotName::People), Some(&SlotDataValue::Number(5)));
    }

    #[test]
    fn affirmative_confirmation_completes_workflow() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4 people"),
            ],
        )
        .updated_conversation;
        let result = handle(conversation, IntentId::Affirmative, "", vec![]);
        assert!(result.reply.contains("REST-NEW123"));
        assert!(result.updated_conversation.is_idle());
        assert_eq!(result.updated_conversation.known_customer_name(), Some("Alice"));
        assert_eq!(result.updated_conversation.last_reservation_reference(), Some("REST-NEW123"));
    }

    #[test]
    fn past_date_triggers_constraint_error_and_re_prompts() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let result = handle(
            conversation,
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2000-01-01"),
            ],
        );
        assert_eq!(result.reply, "That date is in the past. Please provide a future date.");
        assert!(result.updated_conversation.has_active_workflow());
        assert!(result.updated_conversation.active_workflow().unwrap().slot_value(SlotName::Date).is_none());
    }

    #[test]
    fn people_out_of_range_triggers_constraint_error() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let result = handle(
            conversation,
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "50"),
            ],
        );
        assert!(!result.reply.contains("Do you confirm"));
        assert!(result.updated_conversation.has_active_workflow());
    }

    #[test]
    fn no_availability_clears_date_and_time_slots_and_suggests_next() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4 people"),
            ],
        )
        .updated_conversation;
        let result = ReservationCreateIntentHandler::new(Arc::new(FullStubReservationPort))
            .handle(IntentHandlerInput {
                conversation,
                analysis_intent: &IntentId::Affirmative,
                text: "",
                analysis_entities: &[],
            });
        assert!(result.reply.contains("Monday June 1 at 21:00"));
        assert!(result.updated_conversation.has_active_workflow());
        let wf = result.updated_conversation.active_workflow().unwrap();
        assert!(wf.slot_value(SlotName::Date).is_none());
        assert!(wf.slot_value(SlotName::Time).is_none());
    }

    #[test]
    fn closed_restaurant_clears_date_and_time_and_reports_closed() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4 people"),
            ],
        )
        .updated_conversation;
        let result = ReservationCreateIntentHandler::new(Arc::new(ClosedStubReservationPort))
            .handle(IntentHandlerInput {
                conversation,
                analysis_intent: &IntentId::Affirmative,
                text: "",
                analysis_entities: &[],
            });

        assert!(result.reply.contains("closed") || result.reply.contains("opening hours") || result.reply.contains("horaires"));
        assert!(result.updated_conversation.has_active_workflow());
        let wf = result.updated_conversation.active_workflow().unwrap();
        assert!(wf.slot_value(SlotName::Date).is_none());
        assert!(wf.slot_value(SlotName::Time).is_none());
    }
}
