use std::collections::HashMap;

/// Slot type for validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotType {
    Text,
    Date,
    Time,
    Number,
    Boolean,
}

/// Validated slot value.
#[derive(Debug, Clone, PartialEq)]
pub enum SlotValue {
    Text(String),
    Date(String),
    Time(String),
    Number(u32),
    Boolean(bool),
}

impl SlotValue {
    /// Check if this value matches the expected type.
    pub fn matches_type(&self, slot_type: SlotType) -> bool {
        matches!(
            (self, slot_type),
            (SlotValue::Text(_), SlotType::Text)
                | (SlotValue::Date(_), SlotType::Date)
                | (SlotValue::Time(_), SlotType::Time)
                | (SlotValue::Number(_), SlotType::Number)
                | (SlotValue::Boolean(_), SlotType::Boolean)
        )
    }
}

/// Error when filling a slot.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotError {
    pub slot: String,
    pub message: String,
}

/// Container for filled slots with validation.
#[derive(Debug, Clone, Default)]
pub struct SlotBag {
    slots: HashMap<String, SlotValue>,
}

impl SlotBag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fill a slot. Returns Err if value type does not match or validation fails.
    pub fn fill(
        &mut self,
        name: &str,
        expected_type: SlotType,
        value: SlotValue,
    ) -> Result<(), SlotError> {
        if !value.matches_type(expected_type) {
            return Err(SlotError {
                slot: name.to_string(),
                message: format!("Expected {:?}, got {:?}", expected_type, value),
            });
        }
        self.validate(name, &value)?;
        self.slots.insert(name.to_string(), value);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&SlotValue> {
        self.slots.get(name)
    }

    pub fn is_filled(&self, name: &str) -> bool {
        self.slots.contains_key(name)
    }

    pub fn filled_count(&self) -> usize {
        self.slots.len()
    }

    /// Domain-specific validation rules.
    fn validate(&self, name: &str, value: &SlotValue) -> Result<(), SlotError> {
        match (name, value) {
            ("name", SlotValue::Text(s)) if s.trim().is_empty() || s.len() > 100 => {
                Err(SlotError {
                    slot: name.to_string(),
                    message: "Name must be non-empty (max 100 chars)".to_string(),
                })
            }
            ("people", SlotValue::Number(n)) if *n < 1 || *n > 20 => Err(SlotError {
                slot: name.to_string(),
                message: "People must be between 1 and 20".to_string(),
            }),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_valid_text_slot() {
        let mut bag = SlotBag::new();
        assert!(
            bag.fill("name", SlotType::Text, SlotValue::Text("Alice".into()))
                .is_ok()
        );
        assert!(bag.is_filled("name"));
    }

    #[test]
    fn reject_wrong_type() {
        let mut bag = SlotBag::new();
        assert!(
            bag.fill("name", SlotType::Text, SlotValue::Number(42))
                .is_err()
        );
    }

    #[test]
    fn reject_empty_name() {
        let mut bag = SlotBag::new();
        assert!(
            bag.fill("name", SlotType::Text, SlotValue::Text("".into()))
                .is_err()
        );
    }

    #[test]
    fn reject_people_out_of_range() {
        let mut bag = SlotBag::new();
        assert!(
            bag.fill("people", SlotType::Number, SlotValue::Number(0))
                .is_err()
        );
        assert!(
            bag.fill("people", SlotType::Number, SlotValue::Number(21))
                .is_err()
        );
    }

    #[test]
    fn accept_people_in_range() {
        let mut bag = SlotBag::new();
        assert!(
            bag.fill("people", SlotType::Number, SlotValue::Number(4))
                .is_ok()
        );
    }
}
