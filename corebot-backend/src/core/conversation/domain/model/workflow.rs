use crate::core::conversation::domain::model::intent::{IntentId, IntentPolicy, NluTask};
use crate::core::conversation::domain::model::slot::{
    SlotBag, SlotDefinition, SlotError, SlotName, SlotType, SlotValue,
};

/// An active multi-turn workflow collecting slots toward completion.
/// Always ends with a mandatory confirmation step.
///
/// Lifecycle: Idle -> Workflow(collect slots) -> confirmation -> execute -> Idle
/// Can be cancelled at any time -> Idle
#[derive(Debug, Clone)]
pub struct Workflow {
    pub intent: IntentId,
    pub nlu_task: Option<NluTask>,
    data_slots: Vec<SlotDefinition>,
    slots: SlotBag,
}

impl Workflow {
    /// Create a workflow from the handler-owned policy.
    /// Confirmation is always added automatically at the end.
    pub fn from_policy(policy: &IntentPolicy) -> Self {
        Self {
            intent: policy.id.clone(),
            nlu_task: policy.nlu_task,
            data_slots: policy.workflow_slots.clone(),
            slots: SlotBag::new(),
        }
    }

    pub fn slot_definitions(&self) -> &[SlotDefinition] {
        &self.data_slots
    }

    pub fn slot_value(&self, slot_name: SlotName) -> Option<&SlotValue> {
        self.slots.get(slot_name)
    }

    /// The next slot to collect.
    /// Data slots first (in order), then confirmation.
    pub fn next_required_slot(&self) -> Option<NextSlot<'_>> {
        // First: unfilled required data slots
        for def in &self.data_slots {
            if def.required && !self.slots.is_filled(def.name) {
                return Some(NextSlot::Data(def));
            }
        }

        // Then: confirmation (only when all data slots are filled)
        if !self.slots.is_filled(SlotName::Confirmation) {
            return Some(NextSlot::Confirmation);
        }

        None
    }

    /// Return a new workflow with one validated slot value applied.
    pub fn with_slot(&self, slot_name: SlotName, value: SlotValue) -> Result<Workflow, SlotError> {
        let expected_type = if slot_name == SlotName::Confirmation {
            SlotType::Boolean
        } else {
            self.data_slots
                .iter()
                .find(|slot| slot.name == slot_name)
                .map(|slot| slot.slot_type)
                .ok_or_else(|| SlotError {
                    slot: slot_name,
                    message: format!("Unknown slot: {}", slot_name),
                })?
        };

        let mut updated_workflow = self.clone();
        updated_workflow.slots = self.slots.with_slot(slot_name, expected_type, value)?;
        Ok(updated_workflow)
    }

    /// True when all data slots are filled but confirmation is not yet.
    pub fn is_ready_for_confirmation(&self) -> bool {
        let all_data_filled = self
            .data_slots
            .iter()
            .filter(|s| s.required)
            .all(|s| self.slots.is_filled(s.name));

        all_data_filled && !self.slots.is_filled(SlotName::Confirmation)
    }
}

/// What the conversation engine should ask for next.
#[derive(Debug)]
pub enum NextSlot<'a> {
    /// A regular data slot to collect.
    Data(&'a SlotDefinition),
    /// All data collected, ask for final confirmation.
    Confirmation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::conversation::domain::model::intent::{IntentKind, NluTask, i18n_key};
    use crate::core::conversation::domain::slot::EntityType;

    fn book_workflow() -> Workflow {
        Workflow::from_policy(&IntentPolicy {
            id: IntentId::ReservationCreate,
            kind: IntentKind::Workflow,
            nlu_task: Some(NluTask::ReservationCreate),
            workflow_slots: vec![
                slot(SlotName::Name, SlotType::Text, true),
                slot(SlotName::Date, SlotType::Date, true),
                slot(SlotName::Time, SlotType::Time, true),
                slot(SlotName::People, SlotType::Number, true),
            ],
            confirmation_prompt: None,
            completion_response: None,
        })
    }

    fn cancel_workflow() -> Workflow {
        Workflow::from_policy(&IntentPolicy {
            id: IntentId::ReservationCancel,
            kind: IntentKind::Workflow,
            nlu_task: Some(NluTask::ReservationCancel),
            workflow_slots: vec![
                slot(SlotName::Reference, SlotType::Text, true),
                slot(SlotName::Name, SlotType::Text, false),
                slot(SlotName::Date, SlotType::Date, false),
            ],
            confirmation_prompt: None,
            completion_response: None,
        })
    }

    fn slot(name: SlotName, slot_type: SlotType, required: bool) -> SlotDefinition {
        SlotDefinition {
            name,
            slot_type,
            required,
            entity_types: vec![EntityType::Unknown("test".to_string())],
            prompt: i18n_key("test.prompt"),
        }
    }

    #[test]
    fn book_first_slot_is_name() {
        let wf = book_workflow();
        assert!(
            matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == SlotName::Name)
        );
    }

    #[test]
    fn cancel_goes_straight_to_confirmation() {
        let wf = cancel_workflow();
        assert!(
            matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == SlotName::Reference)
        );
    }

    #[test]
    fn slots_advance_in_order() {
        let mut wf = book_workflow()
            .with_slot(SlotName::Name, SlotValue::Text("Alice".into()))
            .unwrap();
        assert!(
            matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == SlotName::Date)
        );

        wf = wf
            .with_slot(SlotName::Date, SlotValue::Date("2026-06-01".into()))
            .unwrap();
        assert!(
            matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == SlotName::Time)
        );

        wf = wf
            .with_slot(SlotName::Time, SlotValue::Time("19:00".into()))
            .unwrap();
        assert!(
            matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == SlotName::People)
        );
    }

    #[test]
    fn confirmation_only_after_all_data() {
        let wf = book_workflow()
            .with_slot(SlotName::Name, SlotValue::Text("Alice".into()))
            .unwrap()
            .with_slot(SlotName::Date, SlotValue::Date("2026-06-01".into()))
            .unwrap()
            .with_slot(SlotName::Time, SlotValue::Time("19:00".into()))
            .unwrap()
            .with_slot(SlotName::People, SlotValue::Number(4))
            .unwrap();

        assert!(wf.is_ready_for_confirmation());
        assert!(matches!(
            wf.next_required_slot(),
            Some(NextSlot::Confirmation)
        ));
    }

    #[test]
    fn complete_after_confirmation() {
        let wf = book_workflow()
            .with_slot(SlotName::Name, SlotValue::Text("Alice".into()))
            .unwrap()
            .with_slot(SlotName::Date, SlotValue::Date("2026-06-01".into()))
            .unwrap()
            .with_slot(SlotName::Time, SlotValue::Time("19:00".into()))
            .unwrap()
            .with_slot(SlotName::People, SlotValue::Number(4))
            .unwrap()
            .with_slot(SlotName::Confirmation, SlotValue::Boolean(true))
            .unwrap();

        assert!(wf.next_required_slot().is_none());
    }

    #[test]
    fn unknown_slot_rejected() {
        assert!(
            book_workflow()
                .with_slot(SlotName::Allergen, SlotValue::Text("blue".into()))
                .is_err()
        );
    }

    #[test]
    fn wrong_type_rejected() {
        assert!(
            book_workflow()
                .with_slot(SlotName::People, SlotValue::Text("four".into()))
                .is_err()
        );
    }
}
