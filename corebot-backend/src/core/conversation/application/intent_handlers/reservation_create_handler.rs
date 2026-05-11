use std::sync::Arc;
use chrono::Local;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult, WorkflowPostProcessResult,
};
use crate::core::conversation::domain::date_resolver::{DateResolveError, DateResolver};
use crate::core::conversation::domain::model::intent::{
    IntentId, IntentKind, IntentPolicy, NluTask, i18n_key,
};
use crate::core::conversation::domain::slot::{EntityType, SlotDefinition, SlotName, SlotType, SlotValue};

pub struct ReservationCreateIntentHandler {
    date_resolver: Arc<dyn DateResolver>,
}

impl ReservationCreateIntentHandler {
    pub fn new(date_resolver: Arc<dyn DateResolver>) -> Self {
        Self { date_resolver }
    }
}

impl IntentHandler for ReservationCreateIntentHandler {
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

    fn post_process(
        &self,
        lang: &str,
        conversation: &crate::core::conversation::domain::conversation::Conversation,
    ) -> WorkflowPostProcessResult {
        // Resolve and validate the date slot if present.
        if let Some(workflow) = conversation.active_workflow() {
            if let Some(SlotValue::Date(raw_date)) = workflow.slot_value(SlotName::Date) {
                let today = Local::now().date_naive();
                match self.date_resolver.resolve(raw_date, today) {
                    Ok(_) => {} // date is valid and in the future
                    Err(DateResolveError::PastDate(_)) => {
                        return WorkflowPostProcessResult::Failed {
                            reply: rust_i18n::t!("workflow.reservation_create.past_date.error", locale = lang).to_string(),
                        };
                    }
                    Err(DateResolveError::Unparseable) => {
                        return WorkflowPostProcessResult::Failed {
                            reply: rust_i18n::t!("workflow.reservation_create.past_date.error", locale = lang).to_string(),
                        };
                    }
                }
            }
        }
        WorkflowPostProcessResult::Succeeded { reply: None }
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
    use std::sync::Arc;
    use super::*;
    use crate::core::conversation::domain::conversation::Conversation;
    use crate::core::conversation::domain::domain_type::DomainType;
    use crate::core::conversation::domain::slot::SlotValue;
    use crate::core::nlu_engine::domain::analysis::NluEntity;

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
            fn resolve(&self, _raw: &str, today: chrono::NaiveDate) -> Result<chrono::NaiveDate, crate::core::conversation::domain::date_resolver::DateResolveError> {
                Ok(today + chrono::Duration::days(1))
            }
        }
        ReservationCreateIntentHandler::new(Arc::new(AlwaysOk)).handle(IntentHandlerInput {
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
            "I have the reservation details. Do you confirm this reservation?"
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
            "I have the reservation details. Do you confirm this reservation?"
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

        assert_eq!(result.reply, "Your reservation request is confirmed.");
        assert!(result.updated_conversation.is_idle());
    }
}
