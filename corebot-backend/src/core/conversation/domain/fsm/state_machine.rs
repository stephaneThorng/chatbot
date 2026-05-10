use crate::core::conversation::domain::catalog::intent::{
    IntentId, IntentKind, IntentPolicy, NluTask,
};
use crate::core::conversation::domain::model::conversation::Conversation;
use crate::core::conversation::domain::model::domain_type::DomainType;
use crate::core::conversation::domain::model::slot::{
    EntityType, SlotDefinition, SlotName, SlotType, SlotValue,
};
use crate::core::conversation::domain::model::workflow::NextSlot;

/// NLU inference context derived from the current conversation state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NluContext {
    pub lang: String,
    pub domain: DomainType,
    pub task: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DetectedEntity {
    pub entity_type: EntityType,
    pub value: String,
}

/// NLU result shape consumed by the workflow state machine.
///
/// This keeps the domain FSM independent from the full NLU engine result,
/// confidence scores, token labels, and preprocessing details.
#[derive(Debug, Clone, PartialEq)]
pub struct ConversationInput {
    pub intent: IntentId,
    pub entities: Vec<DetectedEntity>,
    pub policy: Option<IntentPolicy>,
}

/// Typed outcome emitted by workflow state transitions.
///
/// Application code renders these effects to user-visible text. The FSM does
/// not call i18n, gateways, or intent handlers.
#[derive(Debug, Clone, PartialEq)]
pub enum ConversationEffect {
    SystemText {
        key: String,
        params: Vec<(String, String)>,
    },
    SlotPrompt {
        workflow_intent: IntentId,
        slot_name: SlotName,
    },
    ConfirmationPrompt {
        workflow_intent: IntentId,
    },
    WorkflowCompletion {
        workflow_intent: IntentId,
    },
}

#[derive(Debug, Clone)]
pub struct StateMachineResult {
    pub updated_conversation: Conversation,
    pub effect: ConversationEffect,
}

/// Pure domain state machine for persistent workflow transitions.
pub struct ConversationStateMachine;

impl ConversationStateMachine {
    pub fn apply(conversation: &Conversation, input: ConversationInput) -> StateMachineResult {
        let mut updated_conversation = conversation.clone();
        let ConversationInput {
            intent,
            entities,
            policy,
        } = input;

        let effect = if updated_conversation.has_active_workflow() {
            Self::apply_active(&mut updated_conversation, intent, entities)
        } else {
            Self::apply_idle(&mut updated_conversation, intent, entities, policy)
        };

        StateMachineResult {
            updated_conversation,
            effect,
        }
    }

    pub fn detect_task(conversation: &Conversation) -> Option<NluTask> {
        let workflow = conversation.active_workflow()?;
        if workflow.is_ready_for_confirmation() {
            return Some(NluTask::Choice);
        }

        workflow.nlu_task
    }

    fn apply_idle(
        conversation: &mut Conversation,
        intent: IntentId,
        entities: Vec<DetectedEntity>,
        policy: Option<IntentPolicy>,
    ) -> ConversationEffect {
        if policy
            .as_ref()
            .is_some_and(|policy| policy.kind == IntentKind::Workflow)
        {
            let policy = policy.expect("workflow policy checked above");
            let _ = conversation.start_workflow(&policy);
            Self::fill_slots_from_entities(conversation, &entities);
            return Self::workflow_state_effect(conversation);
        }

        Self::system_text("echo_intent", vec![("intent", intent.as_str())])
    }

    fn apply_active(
        conversation: &mut Conversation,
        intent: IntentId,
        entities: Vec<DetectedEntity>,
    ) -> ConversationEffect {
        if intent == IntentId::Cancel {
            conversation.cancel_workflow();
            return Self::system_text("workflow_cancelled", vec![]);
        }

        if conversation
            .active_workflow()
            .is_some_and(|workflow| workflow.is_ready_for_confirmation())
        {
            return Self::apply_confirmation(conversation, intent);
        }

        Self::fill_slots_from_entities(conversation, &entities);
        Self::workflow_state_effect(conversation)
    }

    fn apply_confirmation(conversation: &mut Conversation, intent: IntentId) -> ConversationEffect {
        match intent {
            IntentId::Affirmative => {
                if let Some(workflow) = conversation.active_workflow_mut() {
                    let _ = workflow.fill_slot(SlotName::Confirmation, SlotValue::Boolean(true));
                    let completed_intent = workflow.intent.clone();
                    conversation.complete_workflow();
                    return ConversationEffect::WorkflowCompletion {
                        workflow_intent: completed_intent,
                    };
                }
                Self::system_text("no_active_workflow", vec![])
            }
            IntentId::Negative => {
                conversation.cancel_workflow();
                Self::system_text("workflow_cancelled", vec![])
            }
            _ => Self::system_text("confirm_yes_no", vec![]),
        }
    }

    fn workflow_state_effect(conversation: &Conversation) -> ConversationEffect {
        let Some(workflow) = conversation.active_workflow() else {
            return Self::system_text("no_active_workflow", vec![]);
        };

        match workflow.next_required_slot() {
            Some(NextSlot::Data(definition)) => ConversationEffect::SlotPrompt {
                workflow_intent: workflow.intent.clone(),
                slot_name: definition.name.clone(),
            },
            Some(NextSlot::Confirmation) => ConversationEffect::ConfirmationPrompt {
                workflow_intent: workflow.intent.clone(),
            },
            None => Self::system_text("workflow_complete", vec![]),
        }
    }

    fn fill_slots_from_entities(conversation: &mut Conversation, entities: &[DetectedEntity]) {
        let Some(workflow) = conversation.active_workflow_mut() else {
            return;
        };
        let slot_definitions = workflow.slot_definitions().to_vec();

        for entity in entities {
            for slot in &slot_definitions {
                if !slot
                    .entity_types
                    .iter()
                    .any(|entity_type| entity_type == &entity.entity_type)
                {
                    continue;
                }
                if let Some(slot_value) = Self::slot_value_from_entity(slot, entity) {
                    let _ = workflow.fill_slot(slot.name, slot_value);
                }
            }
        }
    }

    fn slot_value_from_entity(slot: &SlotDefinition, entity: &DetectedEntity) -> Option<SlotValue> {
        match slot.slot_type {
            SlotType::Text => Some(SlotValue::Text(entity.value.clone())),
            SlotType::Date => Some(SlotValue::Date(entity.value.clone())),
            SlotType::Time => Some(SlotValue::Time(entity.value.clone())),
            SlotType::Number => Self::parse_people_count(entity).map(SlotValue::Number),
            SlotType::Boolean => None,
        }
    }

    fn parse_people_count(entity: &DetectedEntity) -> Option<u32> {
        let digits = entity
            .value
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>();
        digits.parse().ok()
    }

    fn system_text(key: &str, params: Vec<(&str, &str)>) -> ConversationEffect {
        ConversationEffect::SystemText {
            key: key.to_string(),
            params: params
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::conversation::domain::conversation::Conversation;
    use crate::core::conversation::domain::intent::i18n_key;
    use crate::core::conversation::domain::slot::SlotValue;

    fn entity(entity_type: EntityType, value: &str) -> DetectedEntity {
        DetectedEntity {
            entity_type,
            value: value.to_string(),
        }
    }

    fn reservation_create_policy() -> IntentPolicy {
        IntentPolicy {
            id: IntentId::ReservationCreate,
            kind: IntentKind::Workflow,
            nlu_task: Some(NluTask::ReservationCreate),
            workflow_slots: vec![
                slot(SlotName::Name, SlotType::Text, EntityType::Person, true),
                slot(SlotName::Date, SlotType::Date, EntityType::Date, true),
                slot(SlotName::Time, SlotType::Time, EntityType::Time, true),
                slot(
                    SlotName::People,
                    SlotType::Number,
                    EntityType::PeopleCount,
                    true,
                ),
            ],
            supported_entities: vec![],
            confirmation_prompt: None,
            completion_response: None,
        }
    }

    fn slot(
        name: SlotName,
        slot_type: SlotType,
        entity_type: EntityType,
        required: bool,
    ) -> SlotDefinition {
        SlotDefinition {
            name,
            slot_type,
            required,
            entity_types: vec![entity_type],
            prompt: i18n_key("test.prompt"),
        }
    }

    fn apply(
        conversation: &Conversation,
        intent: &str,
        entities: Vec<DetectedEntity>,
    ) -> StateMachineResult {
        let intent = IntentId::from(intent);
        let policy = if intent == IntentId::ReservationCreate {
            Some(reservation_create_policy())
        } else {
            None
        };
        ConversationStateMachine::apply(
            conversation,
            ConversationInput {
                intent,
                entities,
                policy,
            },
        )
    }

    fn fill_ready_reservation(conversation: &Conversation) -> Conversation {
        apply(
            conversation,
            "reservation_create",
            vec![
                entity(EntityType::Person, "Alice"),
                entity(EntityType::Date, "July 9"),
                entity(EntityType::Time, "7pm"),
                entity(EntityType::PeopleCount, "4"),
            ],
        )
        .updated_conversation
    }

    #[test]
    fn idle_reservation_create_starts_workflow() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        let result = apply(&conversation, "reservation_create", vec![]);
        conversation = result.updated_conversation;

        assert!(conversation.has_active_workflow());
        assert!(matches!(
            result.effect,
            ConversationEffect::SlotPrompt { slot_name, .. } if slot_name == SlotName::Name
        ));
    }

    #[test]
    fn active_workflow_derives_reservation_create_task() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        conversation = apply(&conversation, "reservation_create", vec![]).updated_conversation;

        let result = ConversationStateMachine::detect_task(&conversation);

        assert_eq!(result, Some(NluTask::ReservationCreate));
    }

    #[test]
    fn ready_workflow_derives_choice_task() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        conversation = fill_ready_reservation(&conversation);

        let result = ConversationStateMachine::detect_task(&conversation);

        assert_eq!(result, Some(NluTask::Choice));
    }

    #[test]
    fn slot_entities_fill_matching_catalog_slots() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        conversation = apply(
            &conversation,
            "reservation_create",
            vec![entity(EntityType::Person, "Alice")],
        )
        .updated_conversation;

        let workflow = conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slots.get(SlotName::Name),
            Some(&SlotValue::Text("Alice".to_string()))
        );
    }

    #[test]
    fn affirmative_completes_workflow() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        conversation = fill_ready_reservation(&conversation);

        let result = apply(&conversation, "affirmative", vec![]);
        conversation = result.updated_conversation;

        assert!(conversation.is_idle());
        assert!(matches!(
            result.effect,
            ConversationEffect::WorkflowCompletion { workflow_intent }
                if workflow_intent == IntentId::from("reservation_create")
        ));
    }

    #[test]
    fn negative_cancels_workflow() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        conversation = fill_ready_reservation(&conversation);

        let result = apply(&conversation, "negative", vec![]);
        conversation = result.updated_conversation;

        assert!(conversation.is_idle());
        assert!(matches!(
            result.effect,
            ConversationEffect::SystemText { key, .. } if key == "workflow_cancelled"
        ));
    }

    #[test]
    fn cancel_interrupts_active_workflow() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        conversation = apply(&conversation, "reservation_create", vec![]).updated_conversation;

        let result = apply(&conversation, "cancel", vec![]);
        conversation = result.updated_conversation;

        assert!(conversation.is_idle());
        assert!(matches!(
            result.effect,
            ConversationEffect::SystemText { key, .. } if key == "workflow_cancelled"
        ));
    }

    #[test]
    fn unknown_non_workflow_intent_in_idle_produces_fallback_effect() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = apply(&conversation, "not_in_catalog", vec![]);

        assert!(conversation.is_idle());
        assert!(matches!(
            result.effect,
            ConversationEffect::SystemText { key, .. } if key == "echo_intent"
        ));
    }

    #[test]
    fn hotel_empty_catalog_does_not_start_restaurant_workflows() {
        let conversation = Conversation::new(DomainType::Hotel);
        let effect = ConversationStateMachine::apply(
            &conversation,
            ConversationInput {
                intent: IntentId::from("reservation_create"),
                entities: vec![],
                policy: None,
            },
        )
        .effect;

        assert!(conversation.is_idle());
        assert!(matches!(
            effect,
            ConversationEffect::SystemText { key, .. } if key == "echo_intent"
        ));
    }
}
