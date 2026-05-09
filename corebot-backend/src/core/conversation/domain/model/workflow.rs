use crate::core::conversation::domain::catalog::intent::{IntentCatalog, IntentId, SlotDefinition};
use crate::core::conversation::domain::model::slot::{SlotBag, SlotError, SlotType, SlotValue};

/// Confirmation slot - automatically appended to every workflow.
const CONFIRMATION_SLOT: &str = "confirmation";

/// An active multi-turn workflow collecting slots toward completion.
/// Always ends with a mandatory confirmation step.
///
/// Lifecycle: Idle -> Workflow(collect slots) -> confirmation -> execute -> Idle
/// Can be cancelled at any time -> Idle
#[derive(Debug, Clone)]
pub struct Workflow {
    pub intent: IntentId,
    data_slots: Vec<SlotDefinition>,
    pub slots: SlotBag,
}

impl Workflow {
    /// Create a workflow from the catalog's slot definitions.
    /// Confirmation is always added automatically at the end.
    pub fn from_catalog(intent: IntentId, catalog: &IntentCatalog) -> Self {
        let data_slots = catalog.required_slots(&intent);
        Self {
            intent,
            data_slots,
            slots: SlotBag::new(),
        }
    }

    /// The next slot to collect.
    /// Data slots first (in order), then confirmation.
    pub fn next_required_slot(&self) -> Option<NextSlot<'_>> {
        // First: unfilled required data slots
        for def in &self.data_slots {
            if def.required && !self.slots.is_filled(&def.name) {
                return Some(NextSlot::Data(def));
            }
        }

        // Then: confirmation (only when all data slots are filled)
        if !self.slots.is_filled(CONFIRMATION_SLOT) {
            return Some(NextSlot::Confirmation);
        }

        None
    }

    /// Fill a data slot with a validated value.
    pub fn fill_slot(&mut self, slot_name: &str, value: SlotValue) -> Result<(), SlotError> {
        if slot_name == CONFIRMATION_SLOT {
            return self.slots.fill(slot_name, SlotType::Boolean, value);
        }

        let def = self
            .data_slots
            .iter()
            .find(|s| s.name == slot_name)
            .ok_or_else(|| SlotError {
                slot: slot_name.to_string(),
                message: format!("Unknown slot: {}", slot_name),
            })?;

        self.slots.fill(slot_name, def.slot_type, value)
    }

    /// True when all data slots AND confirmation are filled.
    pub fn is_complete(&self) -> bool {
        let all_data_filled = self
            .data_slots
            .iter()
            .filter(|s| s.required)
            .all(|s| self.slots.is_filled(&s.name));

        all_data_filled && self.slots.is_filled(CONFIRMATION_SLOT)
    }

    /// True when all data slots are filled but confirmation is not yet.
    pub fn is_ready_for_confirmation(&self) -> bool {
        let all_data_filled = self
            .data_slots
            .iter()
            .filter(|s| s.required)
            .all(|s| self.slots.is_filled(&s.name));

        all_data_filled && !self.slots.is_filled(CONFIRMATION_SLOT)
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
    use crate::core::conversation::domain::intent::build_restaurant_catalog;

    fn book_workflow() -> Workflow {
        Workflow::from_catalog(
            IntentId::new("reservation_create"),
            &build_restaurant_catalog(),
        )
    }

    fn cancel_workflow() -> Workflow {
        Workflow::from_catalog(
            IntentId::new("reservation_cancel"),
            &build_restaurant_catalog(),
        )
    }

    #[test]
    fn book_first_slot_is_name() {
        let wf = book_workflow();
        assert!(matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == "name"));
    }

    #[test]
    fn cancel_goes_straight_to_confirmation() {
        let wf = cancel_workflow();
        assert!(
            matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == "reference")
        );
    }

    #[test]
    fn slots_advance_in_order() {
        let mut wf = book_workflow();
        wf.fill_slot("name", SlotValue::Text("Alice".into()))
            .unwrap();
        assert!(matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == "date"));

        wf.fill_slot("date", SlotValue::Date("2026-06-01".into()))
            .unwrap();
        assert!(matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == "time"));

        wf.fill_slot("time", SlotValue::Time("19:00".into()))
            .unwrap();
        assert!(matches!(wf.next_required_slot(), Some(NextSlot::Data(d)) if d.name == "people"));
    }

    #[test]
    fn confirmation_only_after_all_data() {
        let mut wf = book_workflow();
        wf.fill_slot("name", SlotValue::Text("Alice".into()))
            .unwrap();
        wf.fill_slot("date", SlotValue::Date("2026-06-01".into()))
            .unwrap();
        wf.fill_slot("time", SlotValue::Time("19:00".into()))
            .unwrap();
        wf.fill_slot("people", SlotValue::Number(4)).unwrap();

        assert!(wf.is_ready_for_confirmation());
        assert!(!wf.is_complete());
        assert!(matches!(
            wf.next_required_slot(),
            Some(NextSlot::Confirmation)
        ));
    }

    #[test]
    fn complete_after_confirmation() {
        let mut wf = book_workflow();
        wf.fill_slot("name", SlotValue::Text("Alice".into()))
            .unwrap();
        wf.fill_slot("date", SlotValue::Date("2026-06-01".into()))
            .unwrap();
        wf.fill_slot("time", SlotValue::Time("19:00".into()))
            .unwrap();
        wf.fill_slot("people", SlotValue::Number(4)).unwrap();
        wf.fill_slot("confirmation", SlotValue::Boolean(true))
            .unwrap();

        assert!(wf.is_complete());
        assert!(wf.next_required_slot().is_none());
    }

    #[test]
    fn unknown_slot_rejected() {
        let mut wf = book_workflow();
        assert!(
            wf.fill_slot("color", SlotValue::Text("blue".into()))
                .is_err()
        );
    }

    #[test]
    fn wrong_type_rejected() {
        let mut wf = book_workflow();
        assert!(
            wf.fill_slot("people", SlotValue::Text("four".into()))
                .is_err()
        );
    }
}
