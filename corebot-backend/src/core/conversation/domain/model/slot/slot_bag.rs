use std::collections::HashMap;

use super::{SlotDataType, SlotDataValue, SlotError, SlotName};

/// Container for filled workflow slots.
#[derive(Debug, Clone, Default)]
pub struct SlotBag {
    slots: HashMap<SlotName, SlotDataValue>,
}

impl SlotBag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_slot(
        mut self,
        name: SlotName,
        expected_type: SlotDataType,
        value: SlotDataValue,
    ) -> Result<Self, SlotError> {
        self.set_slot(name, expected_type, value)?;
        Ok(self)
    }

    pub fn set_slot(
        &mut self,
        name: SlotName,
        expected_type: SlotDataType,
        value: SlotDataValue,
    ) -> Result<(), SlotError> {
        if !value.matches_type(expected_type) {
            return Err(SlotError {
                slot: name,
                message: format!("Expected {:?}, got {:?}", expected_type, value),
            });
        }
        self.slots.insert(name, value);
        Ok(())
    }

    pub fn get(&self, name: SlotName) -> Option<&SlotDataValue> {
        self.slots.get(&name)
    }

    pub fn is_filled(&self, name: SlotName) -> bool {
        self.slots.contains_key(&name)
    }

    pub fn remove(&mut self, name: SlotName) {
        self.slots.remove(&name);
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
                SlotDataType::Text,
                SlotDataValue::Text("Alice".into()),
            )
            .unwrap();
        assert!(updated.is_filled(SlotName::Name));
        assert!(!bag.is_filled(SlotName::Name));
    }

    #[test]
    fn reject_wrong_type() {
        let bag = SlotBag::new();
        let result = bag.into_slot(
            SlotName::Name,
            SlotDataType::Text,
            SlotDataValue::Number(42),
        );
        assert!(result.is_err());
    }

    #[test]
    fn remove_clears_slot() {
        let mut bag = SlotBag::new();
        bag.set_slot(
            SlotName::Name,
            SlotDataType::Text,
            SlotDataValue::Text("Alice".into()),
        )
        .unwrap();
        bag.remove(SlotName::Name);
        assert!(!bag.is_filled(SlotName::Name));
    }
}
