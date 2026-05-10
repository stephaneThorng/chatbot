use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult, WorkflowPostProcessResult,
};
use crate::core::conversation::domain::intent::{
    IntentId, IntentKind, IntentPolicy, NluTask, i18n_key,
};
use crate::core::conversation::domain::slot::{EntityType, SlotDefinition, SlotName, SlotType};

pub struct ReservationCreateIntentHandler;

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
            supported_entities: vec![
                EntityType::Person,
                EntityType::Date,
                EntityType::Time,
                EntityType::PeopleCount,
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
        _lang: &str,
        _conversation: &crate::core::conversation::domain::conversation::Conversation,
    ) -> WorkflowPostProcessResult {
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
    use super::*;
    use crate::core::conversation::domain::conversation::Conversation;
    use crate::core::conversation::domain::domain_type::DomainType;
    use crate::core::conversation::domain::slot::SlotValue;
    use crate::core::conversation::domain::state_machine::DetectedEntity;

    fn entity(entity_type: EntityType, value: &str) -> DetectedEntity {
        DetectedEntity {
            entity_type,
            value: value.to_string(),
        }
    }

    fn handle(
        conversation: &Conversation,
        intent: IntentId,
        entities: Vec<DetectedEntity>,
    ) -> StateHandlerResult {
        ReservationCreateIntentHandler.handle(IntentHandlerInput {
            conversation,
            analysis_intent: &intent,
            text: "",
            analysis_entities: &entities,
        })
    }

    #[test]
    fn idle_workflow_prompts_for_first_missing_slot() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = handle(&conversation, IntentId::ReservationCreate, vec![]);

        assert_eq!(result.reply, "What name should I use for the reservation?");
        assert!(result.updated_conversation.has_active_workflow());
    }

    #[test]
    fn missing_slots_are_filled_from_entities() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = handle(
            &conversation,
            IntentId::ReservationCreate,
            vec![
                entity(EntityType::Person, "Alice"),
                entity(EntityType::Date, "June 12"),
                entity(EntityType::Time, "7pm"),
            ],
        );

        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slots.get(SlotName::Name),
            Some(&SlotValue::Text("Alice".to_string()))
        );
        assert_eq!(result.reply, "For how many people?");
    }

    #[test]
    fn filled_workflow_asks_for_confirmation() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = handle(
            &conversation,
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
            &Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            vec![
                entity(EntityType::Person, "Alice"),
                entity(EntityType::Date, "June 12"),
                entity(EntityType::Time, "7pm"),
                entity(EntityType::PeopleCount, "4 people"),
            ],
        )
        .updated_conversation;

        let result = handle(&conversation, IntentId::Negative, vec![]);

        assert_eq!(result.reply, "Okay. What would you like to change?");
        assert!(result.updated_conversation.has_active_workflow());
    }

    #[test]
    fn updated_slot_reasks_for_confirmation() {
        let conversation = handle(
            &Conversation::new(DomainType::Restaurant),
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
            &conversation,
            IntentId::Negative,
            vec![entity(EntityType::PeopleCount, "5 people")],
        );

        assert_eq!(
            result.reply,
            "I have the reservation details. Do you confirm this reservation?"
        );
        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slots.get(SlotName::People),
            Some(&SlotValue::Number(5))
        );
    }

    #[test]
    fn affirmative_confirmation_completes_workflow() {
        let conversation = handle(
            &Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            vec![
                entity(EntityType::Person, "Alice"),
                entity(EntityType::Date, "June 12"),
                entity(EntityType::Time, "7pm"),
                entity(EntityType::PeopleCount, "4 people"),
            ],
        )
        .updated_conversation;

        let result = handle(&conversation, IntentId::Affirmative, vec![]);

        assert_eq!(result.reply, "Your reservation request is confirmed.");
        assert!(result.updated_conversation.is_idle());
    }
}
