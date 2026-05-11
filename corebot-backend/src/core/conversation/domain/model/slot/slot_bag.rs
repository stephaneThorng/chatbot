use std::collections::HashMap;

use super::{SlotError, SlotName, SlotType, SlotValue};

/// Container for filled workflow slots.
///
/// `SlotBag` owns slot validation. Callers receive an updated bag instead of
/// mutating this one directly.
#[derive(Debug, Clone, Default)]
pub struct SlotBag {
    slots: HashMap<SlotName, SlotValue>,
}

impl SlotBag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a new slot bag with the slot filled after type and domain validation.
    pub fn into_slot(
        mut self,
        name: SlotName,
        expected_type: SlotType,
        value: SlotValue,
    ) -> Result<Self, SlotError> {
        self.set_slot(name, expected_type, value)?;
        Ok(self)
    }

    pub fn set_slot(
        &mut self,
        name: SlotName,
        expected_type: SlotType,
        value: SlotValue,
    ) -> Result<(), SlotError> {
        if !value.matches_type(expected_type) {
            return Err(SlotError {
                slot: name,
                message: format!("Expected {:?}, got {:?}", expected_type, value),
            });
        }

        Self::validate(name, &value)?;

        self.slots.insert(name, value);
        Ok(())
    }

    pub fn get(&self, name: SlotName) -> Option<&SlotValue> {
        self.slots.get(&name)
    }

    pub fn is_filled(&self, name: SlotName) -> bool {
        self.slots.contains_key(&name)
    }

    fn validate(name: SlotName, value: &SlotValue) -> Result<(), SlotError> {
        match (name, value) {
            (SlotName::Name, SlotValue::Text(value))
                if value.trim().is_empty() || value.len() > 100 =>
            {
                Err(SlotError {
                    slot: name,
                    message: "Name must be non-empty (max 100 chars)".to_string(),
                })
            }
            (SlotName::People, SlotValue::Number(value)) if *value < 1 || *value > 20 => {
                Err(SlotError {
                    slot: name,
                    message: "People must be between 1 and 20".to_string(),
                })
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_slot_returns_updated_bag() {
        let bag = SlotBag::new();

        let updated = bag
            .clone()
            .into_slot(
                SlotName::Name,
                SlotType::Text,
                SlotValue::Text("Alice".into()),
            )
            .unwrap();

        assert!(updated.is_filled(SlotName::Name));
        assert!(!bag.is_filled(SlotName::Name));
    }

    #[test]
    fn reject_wrong_type() {
        let bag = SlotBag::new();

        let result = bag.into_slot(SlotName::Name, SlotType::Text, SlotValue::Number(42));

        assert!(result.is_err());
    }

    #[test]
    fn reject_empty_name() {
        let bag = SlotBag::new();

        let result = bag.into_slot(SlotName::Name, SlotType::Text, SlotValue::Text("".into()));

        assert!(result.is_err());
    }

    #[test]
    fn reject_people_out_of_range() {
        let bag = SlotBag::new();

        assert!(
            bag.clone()
                .into_slot(SlotName::People, SlotType::Number, SlotValue::Number(0))
                .is_err()
        );
        assert!(
            bag.into_slot(SlotName::People, SlotType::Number, SlotValue::Number(21))
                .is_err()
        );
    }

    #[test]
    fn accept_people_in_range() {
        let bag = SlotBag::new();

        assert!(
            bag.into_slot(SlotName::People, SlotType::Number, SlotValue::Number(4))
                .is_ok()
        );
    }
}
