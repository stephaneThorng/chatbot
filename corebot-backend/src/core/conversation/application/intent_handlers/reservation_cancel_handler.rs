use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::domain::model::intent::{
    IntentId, IntentKind, IntentPolicy, NluTask, i18n_key,
};
use crate::core::conversation::domain::slot::{EntityType, SlotDefinition, SlotName, SlotType};

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
                slot(
                    SlotName::Reference,
                    SlotType::Text,
                    true,
                    vec![EntityType::ReservationReference],
                    "workflow.reservation_cancel.slot.reference.prompt",
                ),
                slot(
                    SlotName::Name,
                    SlotType::Text,
                    false,
                    vec![EntityType::Person],
                    "workflow.reservation_cancel.slot.name.prompt",
                ),
                slot(
                    SlotName::Date,
                    SlotType::Date,
                    false,
                    vec![EntityType::Date],
                    "workflow.reservation_cancel.slot.date.prompt",
                ),
            ],
            confirmation_prompt: Some(i18n_key("workflow.reservation_cancel.confirmation.prompt")),
            completion_response: Some(i18n_key("workflow.reservation_cancel.completion.success")),
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        self.handle_workflow(input)
    }
}

fn slot(
    name: SlotName,
    slot_type: SlotType,
    required: bool,
    entity_types: Vec<EntityType>,
    prompt: &str,
) -> SlotDefinition {
    SlotDefinition {
        name,
        slot_type,
        required,
        entity_types,
        prompt: i18n_key(prompt),
    }
}
