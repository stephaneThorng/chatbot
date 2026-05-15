use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::domain::model::intent::{
    IntentConfig, IntentId, IntentWorkflow, NluTask, WorkflowConfig, i18n_key,
};
use crate::core::conversation::domain::model::slot::{
    SlotConfig, SlotConstraint, SlotConstraintEntry, SlotName,
};

pub struct ReservationCancelIntentHandler;

impl IntentHandler for ReservationCancelIntentHandler {
    fn intent(&self) -> IntentId {
        IntentId::ReservationCancel
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Workflow(WorkflowConfig {
                nlu_task: Some(NluTask::ReservationCancel),
                slots: vec![
                    SlotConfig {
                        name: SlotName::Reference,
                        required: true,
                        prompt: i18n_key("workflow.reservation_cancel.slot.reference.prompt"),
                        constraints: vec![],
                    },
                    SlotConfig {
                        name: SlotName::Name,
                        required: false,
                        prompt: i18n_key("workflow.reservation_cancel.slot.name.prompt"),
                        constraints: vec![],
                    },
                    SlotConfig {
                        name: SlotName::Date,
                        required: false,
                        prompt: i18n_key("workflow.reservation_cancel.slot.date.prompt"),
                        constraints: vec![SlotConstraintEntry::new(SlotConstraint::FutureDate)],
                    },
                ],
                starting_message: None,
                confirmation_prompt: Some(i18n_key(
                    "workflow.reservation_cancel.confirmation.prompt",
                )),
                completion_response: Some(i18n_key(
                    "workflow.reservation_cancel.completion.success",
                )),
            }),
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        self.handle_workflow(input)
    }
}
