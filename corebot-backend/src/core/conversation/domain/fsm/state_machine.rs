use crate::core::conversation::domain::catalog::intent::{
    IntentCatalog, IntentId, NluTask, SlotDefinition,
};
use crate::core::conversation::domain::model::conversation::Conversation;
use crate::core::conversation::domain::model::domain_type::DomainType;
use crate::core::conversation::domain::model::slot::{SlotType, SlotValue};
use crate::core::conversation::domain::model::workflow::NextSlot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NluContext {
    pub lang: String,
    pub domain: DomainType,
    pub task: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DetectedEntity {
    pub entity_type: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationEvent {
    NluAnalysisApplied {
        intent: IntentId,
        text: String,
        entities: Vec<DetectedEntity>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationEffect {
    IntentResponse {
        intent: IntentId,
        text: String,
        entities: Vec<DetectedEntity>,
    },
    SystemText {
        key: String,
        params: Vec<(String, String)>,
    },
    SlotPrompt {
        workflow_intent: IntentId,
        slot_name: String,
    },
    ConfirmationPrompt {
        workflow_intent: IntentId,
    },
    WorkflowCompletion {
        workflow_intent: IntentId,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversationTransition {
    pub effect: ConversationEffect,
}

pub struct ConversationStateMachine;

impl ConversationStateMachine {
    pub fn nlu_context(conversation: &Conversation, catalog: &IntentCatalog) -> NluContext {
        NluContext {
            lang: conversation.lang.clone(),
            domain: conversation.domain,
            task: Self::detect_task(conversation, catalog),
        }
    }

    pub fn apply(
        conversation: &mut Conversation,
        catalog: &IntentCatalog,
        event: ConversationEvent,
    ) -> ConversationTransition {
        let ConversationEvent::NluAnalysisApplied {
            intent,
            text,
            entities,
        } = event;

        let effect = if conversation.has_active_workflow() {
            Self::apply_active(conversation, catalog, intent, entities)
        } else {
            Self::apply_idle(conversation, catalog, intent, text, entities)
        };

        ConversationTransition { effect }
    }

    fn detect_task(conversation: &Conversation, catalog: &IntentCatalog) -> Option<String> {
        let workflow = conversation.active_workflow()?;
        if workflow.is_ready_for_confirmation() {
            return Some(NluTask::Choice.as_tag().to_string());
        }

        catalog
            .nlu_task(&workflow.intent)
            .map(|task| task.as_tag().to_string())
    }

    fn apply_idle(
        conversation: &mut Conversation,
        catalog: &IntentCatalog,
        intent: IntentId,
        text: String,
        entities: Vec<DetectedEntity>,
    ) -> ConversationEffect {
        if catalog.is_workflow(&intent) {
            let _ = conversation.start_workflow(&intent, catalog);
            Self::fill_slots_from_entities(conversation, catalog, &entities);
            return Self::workflow_state_effect(conversation);
        }

        if catalog.get(&intent).is_some() {
            return ConversationEffect::IntentResponse {
                intent,
                text,
                entities,
            };
        }

        Self::system_text("echo_intent", vec![("intent", intent.0.as_str())])
    }

    fn apply_active(
        conversation: &mut Conversation,
        catalog: &IntentCatalog,
        intent: IntentId,
        entities: Vec<DetectedEntity>,
    ) -> ConversationEffect {
        if intent.0 == "cancel" {
            conversation.cancel_workflow();
            return Self::system_text("workflow_cancelled", vec![]);
        }

        if conversation
            .active_workflow()
            .is_some_and(|workflow| workflow.is_ready_for_confirmation())
        {
            return Self::apply_confirmation(conversation, intent);
        }

        Self::fill_slots_from_entities(conversation, catalog, &entities);
        Self::workflow_state_effect(conversation)
    }

    fn apply_confirmation(conversation: &mut Conversation, intent: IntentId) -> ConversationEffect {
        match intent.0.as_str() {
            "affirmative" => {
                if let Some(workflow) = conversation.active_workflow_mut() {
                    let _ = workflow.fill_slot("confirmation", SlotValue::Boolean(true));
                    let completed_intent = workflow.intent.clone();
                    conversation.complete_workflow();
                    return ConversationEffect::WorkflowCompletion {
                        workflow_intent: completed_intent,
                    };
                }
                Self::system_text("no_active_workflow", vec![])
            }
            "negative" => {
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

    fn fill_slots_from_entities(
        conversation: &mut Conversation,
        catalog: &IntentCatalog,
        entities: &[DetectedEntity],
    ) {
        let Some(workflow) = conversation.active_workflow_mut() else {
            return;
        };
        let slot_definitions = catalog.required_slots(&workflow.intent);

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
                    let _ = workflow.fill_slot(&slot.name, slot_value);
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
    use crate::core::conversation::domain::intent::build_catalog;
    use crate::core::conversation::domain::slot::SlotValue;

    fn restaurant_catalog() -> IntentCatalog {
        build_catalog(DomainType::Restaurant)
    }

    fn entity(entity_type: &str, value: &str) -> DetectedEntity {
        DetectedEntity {
            entity_type: entity_type.to_string(),
            value: value.to_string(),
        }
    }

    fn apply(
        conversation: &mut Conversation,
        intent: &str,
        entities: Vec<DetectedEntity>,
    ) -> ConversationTransition {
        ConversationStateMachine::apply(
            conversation,
            &restaurant_catalog(),
            ConversationEvent::NluAnalysisApplied {
                intent: IntentId::new(intent),
                text: intent.to_string(),
                entities,
            },
        )
    }

    fn fill_ready_reservation(conversation: &mut Conversation) {
        apply(
            conversation,
            "reservation_create",
            vec![
                entity("person", "Alice"),
                entity("date", "July 9"),
                entity("time", "7pm"),
                entity("people_count", "4"),
            ],
        );
    }

    #[test]
    fn idle_reservation_create_starts_workflow() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        let transition = apply(&mut conversation, "reservation_create", vec![]);

        assert!(conversation.has_active_workflow());
        assert!(matches!(
            transition.effect,
            ConversationEffect::SlotPrompt { slot_name, .. } if slot_name == "name"
        ));
    }

    #[test]
    fn active_workflow_derives_reservation_create_task() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        apply(&mut conversation, "reservation_create", vec![]);

        let context = ConversationStateMachine::nlu_context(&conversation, &restaurant_catalog());

        assert_eq!(context.task, Some("WF_RESERVATION_CREATE".to_string()));
    }

    #[test]
    fn ready_workflow_derives_choice_task() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        fill_ready_reservation(&mut conversation);

        let context = ConversationStateMachine::nlu_context(&conversation, &restaurant_catalog());

        assert_eq!(context.task, Some("WF_CHOICE".to_string()));
    }

    #[test]
    fn slot_entities_fill_matching_catalog_slots() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        apply(
            &mut conversation,
            "reservation_create",
            vec![entity("person", "Alice")],
        );

        let workflow = conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slots.get("name"),
            Some(&SlotValue::Text("Alice".to_string()))
        );
    }

    #[test]
    fn affirmative_completes_workflow() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        fill_ready_reservation(&mut conversation);

        let transition = apply(&mut conversation, "affirmative", vec![]);

        assert!(conversation.is_idle());
        assert!(matches!(
            transition.effect,
            ConversationEffect::WorkflowCompletion { workflow_intent }
                if workflow_intent == IntentId::new("reservation_create")
        ));
    }

    #[test]
    fn negative_cancels_workflow() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        fill_ready_reservation(&mut conversation);

        let transition = apply(&mut conversation, "negative", vec![]);

        assert!(conversation.is_idle());
        assert!(matches!(
            transition.effect,
            ConversationEffect::SystemText { key, .. } if key == "workflow_cancelled"
        ));
    }

    #[test]
    fn cancel_interrupts_active_workflow() {
        let mut conversation = Conversation::new(DomainType::Restaurant);
        apply(&mut conversation, "reservation_create", vec![]);

        let transition = apply(&mut conversation, "cancel", vec![]);

        assert!(conversation.is_idle());
        assert!(matches!(
            transition.effect,
            ConversationEffect::SystemText { key, .. } if key == "workflow_cancelled"
        ));
    }

    #[test]
    fn unknown_non_workflow_intent_in_idle_produces_fallback_effect() {
        let mut conversation = Conversation::new(DomainType::Restaurant);

        let transition = apply(&mut conversation, "not_in_catalog", vec![]);

        assert!(conversation.is_idle());
        assert!(matches!(
            transition.effect,
            ConversationEffect::SystemText { key, .. } if key == "echo_intent"
        ));
    }

    #[test]
    fn hotel_empty_catalog_does_not_start_restaurant_workflows() {
        let mut conversation = Conversation::new(DomainType::Hotel);
        let catalog = build_catalog(DomainType::Hotel);

        let transition = ConversationStateMachine::apply(
            &mut conversation,
            &catalog,
            ConversationEvent::NluAnalysisApplied {
                intent: IntentId::new("reservation_create"),
                text: "reservation_create".to_string(),
                entities: vec![],
            },
        );

        assert!(conversation.is_idle());
        assert!(matches!(
            transition.effect,
            ConversationEffect::SystemText { key, .. } if key == "echo_intent"
        ));
    }
}
