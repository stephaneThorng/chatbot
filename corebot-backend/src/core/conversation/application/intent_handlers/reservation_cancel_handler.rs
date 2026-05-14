use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::domain::model::intent::{
    IntentId, IntentKind, IntentPolicy, NluTask, i18n_key,
};
use crate::core::conversation::domain::slot::{
    EntityType, SlotConstraint, SlotConstraintEntry, SlotDefinition, SlotName, SlotType,
};

pub struct ReservationCancelIntentHandler;

impl IntentHandler for ReservationCancelIntentHandler {
    fn intent(&self) -> IntentId {
        IntentId::ReservationCancel
    }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy {
            id: self.intent(),
            kind: IntentKind::Workflow,
            nlu_task: Some(NluTask::ReservationCancel),
            workflow_slots: vec![
                SlotDefinition {
                    name: SlotName::Reference,
                    slot_type: SlotType::Text,
                    required: true,
                    entity_types: vec![EntityType::ReservationReference],
                    prompt: i18n_key("workflow.reservation_cancel.slot.reference.prompt"),
                    constraints: vec![],
                },
                SlotDefinition {
                    name: SlotName::Name,
                    slot_type: SlotType::Text,
                    required: false,
                    entity_types: vec![EntityType::Person],
                    prompt: i18n_key("workflow.reservation_cancel.slot.name.prompt"),
                    constraints: vec![],
                },
                SlotDefinition {
                    name: SlotName::Date,
                    slot_type: SlotType::Date,
                    required: false,
                    entity_types: vec![EntityType::Date],
                    prompt: i18n_key("workflow.reservation_cancel.slot.date.prompt"),
                    constraints: vec![SlotConstraintEntry::new(SlotConstraint::FutureDate)],
                },
            ],
            starting_message: None,
            confirmation_prompt: Some(i18n_key("workflow.reservation_cancel.confirmation.prompt")),
            completion_response: Some(i18n_key("workflow.reservation_cancel.completion.success")),
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        self.handle_workflow(input)
    }
}
