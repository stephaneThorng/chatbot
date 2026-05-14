use chrono::Local;
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult, WorkflowPostProcessResult,
};
use crate::core::conversation::application::port::outbound::restaurant_queries::ReservationCreateQuery;
use crate::core::conversation::application::port::outbound::restaurant_reservation_port::RestaurantReservationPort;
use crate::core::conversation::domain::date_resolver::{DateResolveError, DateResolver};
use crate::core::conversation::domain::model::conversation::Conversation;
use crate::core::conversation::domain::model::intent::{
    IntentId, IntentKind, IntentPolicy, NluTask, i18n_key,
};
use crate::core::conversation::domain::slot::{
    EntityType, SlotDefinition, SlotName, SlotType, SlotValue,
};

pub struct ReservationCreateIntentHandler<P: RestaurantReservationPort + ?Sized> {
    date_resolver: Arc<dyn DateResolver>,
    reservation_port: Arc<P>,
}

impl<P: RestaurantReservationPort + ?Sized> ReservationCreateIntentHandler<P> {
    pub fn new(date_resolver: Arc<dyn DateResolver>, reservation_port: Arc<P>) -> Self {
        Self {
            date_resolver,
            reservation_port,
        }
    }

    fn workflow_value<'a>(conversation: &'a Conversation, slot: SlotName) -> Option<&'a SlotValue> {
        conversation
            .active_workflow()
            .and_then(|workflow| workflow.slot_value(slot))
    }

    fn workflow_text(conversation: &Conversation, slot: SlotName) -> String {
        match Self::workflow_value(conversation, slot) {
            Some(SlotValue::Text(value)) | Some(SlotValue::Date(value)) | Some(SlotValue::Time(value)) => {
                value.clone()
            }
            Some(SlotValue::Number(value)) => value.to_string(),
            Some(SlotValue::Boolean(value)) => value.to_string(),
            None => String::new(),
        }
    }

    fn workflow_people_count(conversation: &Conversation) -> u32 {
        match Self::workflow_value(conversation, SlotName::People) {
            Some(SlotValue::Number(value)) => *value,
            _ => 0,
        }
    }

    fn confirmation_summary(&self, conversation: &Conversation) -> String {
        rust_i18n::t!(
            "workflow.reservation_create.confirmation.prompt",
            locale = conversation.lang.as_str(),
            name = Self::workflow_text(conversation, SlotName::Name),
            date = Self::workflow_text(conversation, SlotName::Date),
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

    fn policy(&self) -> IntentPolicy {
        IntentPolicy {
            id: self.intent(),
            kind: IntentKind::Workflow,
            nlu_task: Some(NluTask::ReservationCreate),
            workflow_slots: vec![
                required_slot(
                    SlotName::Name,
                    SlotType::Text,
                    vec![EntityType::Person],
                    "workflow.reservation_create.slot.name.prompt",
                ),
                required_slot(
                    SlotName::Date,
                    SlotType::Date,
                    vec![EntityType::Date],
                    "workflow.reservation_create.slot.date.prompt",
                ),
                required_slot(
                    SlotName::Time,
                    SlotType::Time,
                    vec![EntityType::Time],
                    "workflow.reservation_create.slot.time.prompt",
                ),
                required_slot(
                    SlotName::People,
                    SlotType::Number,
                    vec![EntityType::PeopleCount],
                    "workflow.reservation_create.slot.people.prompt",
                ),
            ],
            confirmation_prompt: Some(i18n_key("workflow.reservation_create.confirmation.prompt")),
            completion_response: Some(i18n_key("workflow.reservation_create.completion.success")),
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        self.handle_workflow(input)
    }

    fn negative_prompt(&self, lang: &str) -> String {
        rust_i18n::t!("workflow.reservation_create.update.prompt", locale = lang).to_string()
    }

    fn confirmation_prompt(&self, _policy: &IntentPolicy, conversation: &Conversation) -> String {
        self.confirmation_summary(conversation)
    }

    fn post_process(
        &self,
        lang: &str,
        mut conversation: Conversation,
    ) -> WorkflowPostProcessResult {
        // Resolve and validate the date slot if present.
        if let Some(workflow) = conversation.active_workflow() {
            if let Some(SlotValue::Date(raw_date)) = workflow.slot_value(SlotName::Date) {
                let today = Local::now().date_naive();
                match self.date_resolver.resolve(raw_date, today) {
                    Ok(_) => {} // date is valid and in the future
                    Err(DateResolveError::PastDate(_)) => {
                        return WorkflowPostProcessResult::Failed {
                            updated_conversation: conversation,
                            reply: rust_i18n::t!(
                                "workflow.reservation_create.past_date.error",
                                locale = lang
                            )
                            .to_string(),
                        };
                    }
                    Err(DateResolveError::Unparseable) => {
                        return WorkflowPostProcessResult::Failed {
                            updated_conversation: conversation,
                            reply: rust_i18n::t!(
                                "workflow.reservation_create.past_date.error",
                                locale = lang
                            )
                            .to_string(),
                        };
                    }
                }
            }
        }

        let name = Self::workflow_text(&conversation, SlotName::Name);
        let date = Self::workflow_text(&conversation, SlotName::Date);
        let time = Self::workflow_text(&conversation, SlotName::Time);
        let people_count = Self::workflow_people_count(&conversation);
        let creation = self.reservation_port.create_reservation(ReservationCreateQuery {
            name: name.clone(),
            date: date.clone(),
            time: time.clone(),
            people_count,
        });
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
                    date = date,
                    time = time,
                    people = people_count,
                    reference = reference
                )
                .to_string(),
            ),
        }
    }
}

fn required_slot(
    name: SlotName,
    slot_type: SlotType,
    entity_types: Vec<EntityType>,
    prompt: &str,
) -> SlotDefinition {
    SlotDefinition {
        name,
        slot_type,
        required: true,
        entity_types,
        prompt: i18n_key(prompt),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::conversation::domain::conversation::Conversation;
    use crate::core::conversation::domain::domain_type::DomainType;
    use crate::core::conversation::domain::slot::SlotValue;
    use crate::core::nlu_engine::domain::analysis::NluEntity;
    use std::sync::Arc;

    struct StubReservationPort;

    impl RestaurantReservationPort for StubReservationPort {
        fn create_reservation(&self, _: ReservationCreateQuery) -> String {
            "created:REST-NEW123".to_string()
        }

        fn check_reservation(
            &self,
            _: crate::core::conversation::application::port::outbound::restaurant_queries::ReservationLookupQuery,
        ) -> String {
            "no_reference_or_name:".to_string()
        }
    }

    fn entity(entity_type: EntityType, value: &str) -> NluEntity {
        NluEntity {
            entity_type,
            value: value.to_string(),
            raw_value: value.to_string(),
            start: 0,
            end: value.len(),
            confidence: 1.0,
        }
    }

    fn handle(
        conversation: Conversation,
        intent: IntentId,
        entities: Vec<NluEntity>,
    ) -> StateHandlerResult {
        use crate::core::conversation::domain::date_resolver::DateResolver;
        struct AlwaysOk;
        impl DateResolver for AlwaysOk {
            fn resolve(
                &self,
                _raw: &str,
                today: chrono::NaiveDate,
            ) -> Result<
                chrono::NaiveDate,
                crate::core::conversation::domain::date_resolver::DateResolveError,
            > {
                Ok(today + chrono::Duration::days(1))
            }
        }
        ReservationCreateIntentHandler::new(Arc::new(AlwaysOk), Arc::new(StubReservationPort))
            .handle(IntentHandlerInput {
            conversation,
            analysis_intent: &intent,
            text: "",
            analysis_entities: &entities,
        })
    }

    #[test]
    fn idle_workflow_prompts_for_first_missing_slot() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = handle(conversation, IntentId::ReservationCreate, vec![]);

        assert_eq!(result.reply, "What name should I use for the reservation?");
        assert!(result.updated_conversation.has_active_workflow());
    }

    #[test]
    fn missing_slots_are_filled_from_entities() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = handle(
            conversation,
            IntentId::ReservationCreate,
            vec![
                entity(EntityType::Person, "Alice"),
                entity(EntityType::Date, "June 12"),
                entity(EntityType::Time, "7pm"),
            ],
        );

        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slot_value(SlotName::Name),
            Some(&SlotValue::Text("Alice".to_string()))
        );
        assert_eq!(result.reply, "For how many people?");
    }

    #[test]
    fn filled_workflow_asks_for_confirmation() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = handle(
            conversation,
            IntentId::ReservationCreate,
            vec![
                entity(EntityType::Person, "Alice"),
                entity(EntityType::Date, "June 12"),
                entity(EntityType::Time, "7pm"),
                entity(EntityType::PeopleCount, "4 people"),
            ],
        );

        assert_eq!(
            result.reply,
            "I have the reservation details: Alice, June 12 at 7pm, for 4 people. Do you confirm this reservation?"
        );
    }

    #[test]
    fn bare_numeric_reply_fills_people_slot_when_it_is_next() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            vec![
                entity(EntityType::Person, "Alice"),
                entity(EntityType::Date, "June 12"),
                entity(EntityType::Time, "7pm"),
            ],
        )
        .updated_conversation;

        use crate::core::conversation::domain::date_resolver::DateResolver;
        struct AlwaysOk;
        impl DateResolver for AlwaysOk {
            fn resolve(
                &self,
                _raw: &str,
                today: chrono::NaiveDate,
            ) -> Result<
                chrono::NaiveDate,
                crate::core::conversation::domain::date_resolver::DateResolveError,
            > {
                Ok(today + chrono::Duration::days(1))
            }
        }
        let result =
            ReservationCreateIntentHandler::new(Arc::new(AlwaysOk), Arc::new(StubReservationPort))
                .handle(IntentHandlerInput {
                conversation,
                analysis_intent: &IntentId::ReservationCreate,
                text: "10",
                analysis_entities: &[],
            });

        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(workflow.slot_value(SlotName::People), Some(&SlotValue::Number(10)));
        assert_eq!(
            result.reply,
            "I have the reservation details: Alice, June 12 at 7pm, for 10 people. Do you confirm this reservation?"
        );
    }

    #[test]
    fn negative_confirmation_keeps_workflow_open_for_changes() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            vec![
                entity(EntityType::Person, "Alice"),
                entity(EntityType::Date, "June 12"),
                entity(EntityType::Time, "7pm"),
                entity(EntityType::PeopleCount, "4 people"),
            ],
        )
        .updated_conversation;

        let result = handle(conversation, IntentId::Negative, vec![]);

        assert_eq!(result.reply, "Okay. What would you like to change?");
        assert!(result.updated_conversation.has_active_workflow());
    }

    #[test]
    fn updated_slot_reasks_for_confirmation() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            vec![
                entity(EntityType::Person, "Alice"),
                entity(EntityType::Date, "June 12"),
                entity(EntityType::Time, "7pm"),
                entity(EntityType::PeopleCount, "4 people"),
            ],
        )
        .updated_conversation;

        let result = handle(
            conversation,
            IntentId::Negative,
            vec![entity(EntityType::PeopleCount, "5 people")],
        );

        assert_eq!(
            result.reply,
            "I have the reservation details: Alice, June 12 at 7pm, for 5 people. Do you confirm this reservation?"
        );
        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slot_value(SlotName::People),
            Some(&SlotValue::Number(5))
        );
    }

    #[test]
    fn affirmative_confirmation_completes_workflow() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            vec![
                entity(EntityType::Person, "Alice"),
                entity(EntityType::Date, "June 12"),
                entity(EntityType::Time, "7pm"),
                entity(EntityType::PeopleCount, "4 people"),
            ],
        )
        .updated_conversation;

        let result = handle(conversation, IntentId::Affirmative, vec![]);

        assert_eq!(
            result.reply,
            "Your reservation is confirmed for Alice, June 12 at 7pm, for 4 people. Your reference is REST-NEW123."
        );
        assert!(result.updated_conversation.is_idle());
        assert_eq!(
            result.updated_conversation.known_customer_name(),
            Some("Alice")
        );
        assert_eq!(
            result.updated_conversation.last_reservation_reference(),
            Some("REST-NEW123")
        );
    }
}
