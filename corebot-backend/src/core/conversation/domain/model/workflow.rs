use crate::core::conversation::domain::model::intent::{
    IntentConfig, IntentId, IntentWorkflow, NluTask,
};
use crate::core::conversation::domain::model::slot::{
    SlotBag, SlotConfig, SlotDataValue, SlotError, SlotName,
};

/// An active multi-turn workflow collecting slots toward completion.
///
/// Confirmation is tracked with a dedicated `confirmed` flag instead of a
/// `SlotName::Confirmation` sentinel — the slot name enum only contains
/// domain data slots.
///
/// Lifecycle: Idle → Workflow (collect slots) → confirmation → execute → Idle
/// Can be cancelled at any time → Idle
#[derive(Debug, Clone)]
pub struct Workflow {
    pub intent: IntentId,
    pub nlu_task: Option<NluTask>,
    slot_config: Vec<SlotConfig>,
    slot_bag: SlotBag,
    confirmed: bool,
}

impl Workflow {
    /// Create a workflow from the handler-owned config.
    pub fn from_config(config: &IntentConfig) -> Self {
        let (nlu_task, data_slots) = match &config.workflow {
            IntentWorkflow::Workflow(wf) => (wf.nlu_task, wf.slots.clone()),
            IntentWorkflow::Informational => (None, vec![]),
        };
        Self {
            intent: config.id.clone(),
            nlu_task,
            slot_config: data_slots,
            slot_bag: SlotBag::new(),
            confirmed: false,
        }
    }

    pub fn slot_definitions(&self) -> &[SlotConfig] {
        &self.slot_config
    }

    pub fn slot_value(&self, slot_name: SlotName) -> Option<&SlotDataValue> {
        self.slot_bag.get(slot_name)
    }

    /// The next slot to collect.
    /// Data slots first (in order), then confirmation.
    pub fn next_required_slot(&self) -> Option<NextSlot<'_>> {
        for def in &self.slot_config {
            if def.required && !self.slot_bag.is_filled(def.name) {
                return Some(NextSlot::Data(def));
            }
        }
        if !self.confirmed {
            return Some(NextSlot::Confirmation);
        }
        None
    }

    pub fn into_slot(
        mut self,
        slot_name: SlotName,
        value: SlotDataValue,
    ) -> Result<Workflow, SlotError> {
        self.set_slot(slot_name, value)?;
        Ok(self)
    }

    pub fn set_slot(&mut self, slot_name: SlotName, value: SlotDataValue) -> Result<(), SlotError> {
        let expected_type = self
            .slot_config
            .iter()
            .find(|slot| slot.name == slot_name)
            .map(|slot| slot.name.data_type())
            .ok_or_else(|| SlotError {
                slot: slot_name,
                message: format!("Unknown slot: {}", slot_name),
            })?;
        self.slot_bag.set_slot(slot_name, expected_type, value)?;
        Ok(())
    }

    /// Set the confirmed flag (called when the user says affirmative).
    pub fn confirm(&mut self) {
        self.confirmed = true;
    }

    pub fn reopen_confirmation(&mut self) {
        self.confirmed = false;
    }

    /// Remove a slot value (used to clear an invalid slot after a constraint violation).
    pub fn clear_slot(&mut self, slot_name: SlotName) {
        self.slot_bag.remove(slot_name);
    }

    /// True when all required data slots are filled and confirmation not yet given.
    pub fn is_ready_for_confirmation(&self) -> bool {
        let all_data_filled = self
            .slot_config
            .iter()
            .filter(|s| s.required)
            .all(|s| self.slot_bag.is_filled(s.name));
        all_data_filled && !self.confirmed
    }
}

/// What the conversation engine should ask for next.
#[derive(Debug)]
pub enum NextSlot<'a> {
    /// A regular data slot to collect.
    Data(&'a SlotConfig),
    /// All data collected, ask for final confirmation.
    Confirmation,
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};

    use super::*;
    use crate::core::conversation::domain::model::intent::{
        IntentWorkflow, WorkflowConfig, i18n_key,
    };

    fn book_config() -> IntentConfig {
        IntentConfig {
            id: IntentId::ReservationCreate,
            workflow: IntentWorkflow::Workflow(WorkflowConfig {
                nlu_task: Some(NluTask::ReservationCreate),
                slots: vec![
                    slot(SlotName::Name, true),
                    slot(SlotName::Date, true),
                    slot(SlotName::Time, true),
                    slot(SlotName::People, true),
                ],
                starting_message: None,
                confirmation_prompt: None,
                completion_response: None,
            }),
        }
    }

    fn cancel_config() -> IntentConfig {
        IntentConfig {
            id: IntentId::ReservationCancel,
            workflow: IntentWorkflow::Workflow(WorkflowConfig {
                nlu_task: Some(NluTask::ReservationCancel),
                slots: vec![
                    slot(SlotName::Reference, true),
                    slot(SlotName::Name, false),
                    slot(SlotName::Date, false),
                ],
                starting_message: None,
                confirmation_prompt: None,
                completion_response: None,
            }),
        }
    }

    fn slot(name: SlotName, required: bool) -> SlotConfig {
        SlotConfig {
            name,
            required,
            prompt: i18n_key("test.prompt"),
            constraints: vec![],
        }
    }

    #[test]
    fn book_first_slot_is_name() {
        let wf = Workflow::from_config(&book_config());
        assert!(
            matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == SlotName::Name)
        );
    }

    #[test]
    fn cancel_first_slot_is_reference() {
        let wf = Workflow::from_config(&cancel_config());
        assert!(
            matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == SlotName::Reference)
        );
    }

    #[test]
    fn slots_advance_in_order() {
        let mut wf = Workflow::from_config(&book_config())
            .into_slot(SlotName::Name, SlotDataValue::Text("Alice".into()))
            .unwrap();
        assert!(
            matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == SlotName::Date)
        );
        wf = wf
            .into_slot(
                SlotName::Date,
                SlotDataValue::Date(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            )
            .unwrap();
        assert!(
            matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == SlotName::Time)
        );
        wf = wf
            .into_slot(
                SlotName::Time,
                SlotDataValue::Time(NaiveTime::from_hms_opt(19, 0, 0).unwrap()),
            )
            .unwrap();
        assert!(
            matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == SlotName::People)
        );
    }

    #[test]
    fn confirmation_only_after_all_data() {
        let wf = Workflow::from_config(&book_config())
            .into_slot(SlotName::Name, SlotDataValue::Text("Alice".into()))
            .unwrap()
            .into_slot(
                SlotName::Date,
                SlotDataValue::Date(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            )
            .unwrap()
            .into_slot(
                SlotName::Time,
                SlotDataValue::Time(NaiveTime::from_hms_opt(19, 0, 0).unwrap()),
            )
            .unwrap()
            .into_slot(SlotName::People, SlotDataValue::Number(4))
            .unwrap();
        assert!(wf.is_ready_for_confirmation());
        assert!(matches!(
            wf.next_required_slot(),
            Some(NextSlot::Confirmation)
        ));
    }

    #[test]
    fn complete_after_confirmation() {
        let mut wf = Workflow::from_config(&book_config())
            .into_slot(SlotName::Name, SlotDataValue::Text("Alice".into()))
            .unwrap()
            .into_slot(
                SlotName::Date,
                SlotDataValue::Date(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            )
            .unwrap()
            .into_slot(
                SlotName::Time,
                SlotDataValue::Time(NaiveTime::from_hms_opt(19, 0, 0).unwrap()),
            )
            .unwrap()
            .into_slot(SlotName::People, SlotDataValue::Number(4))
            .unwrap();
        wf.confirm();
        assert!(wf.next_required_slot().is_none());
    }

    #[test]
    fn unknown_slot_rejected() {
        assert!(
            Workflow::from_config(&book_config())
                .into_slot(SlotName::Allergen, SlotDataValue::Text("blue".into()))
                .is_err()
        );
    }

    #[test]
    fn wrong_type_rejected() {
        assert!(
            Workflow::from_config(&book_config())
                .into_slot(SlotName::People, SlotDataValue::Text("four".into()))
                .is_err()
        );
    }
}
