use std::collections::HashMap;
use std::fmt;

use crate::core::conversation::domain::catalog::intent::I18nKey;

/// Compile-time slot names used by workflows and informational handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotName {
    Name,
    Date,
    Time,
    People,
    Reference,
    MenuItem,
    Allergen,
    DietaryRequirement,
    Confirmation,
}

impl SlotName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Date => "date",
            Self::Time => "time",
            Self::People => "people",
            Self::Reference => "reference",
            Self::MenuItem => "menu_item",
            Self::Allergen => "allergen",
            Self::DietaryRequirement => "dietary_requirement",
            Self::Confirmation => "confirmation",
        }
    }
}

impl fmt::Display for SlotName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Entity labels known by the conversation core.
///
/// `Unknown` is only used at the NLU boundary so handlers and workflow logic can
/// avoid string comparisons for supported entity labels.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    Person,
    Date,
    Time,
    PeopleCount,
    ReservationReference,
    MenuItem,
    Allergen,
    DietaryRequirement,
    Unknown(String),
}

impl EntityType {
    pub fn new(label: &str) -> Self {
        match label {
            "person" => Self::Person,
            "date" => Self::Date,
            "time" => Self::Time,
            "people_count" => Self::PeopleCount,
            "reservation_reference" => Self::ReservationReference,
            "menu_item" => Self::MenuItem,
            "allergen" => Self::Allergen,
            "dietary_requirement" => Self::DietaryRequirement,
            value => Self::Unknown(value.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Person => "person",
            Self::Date => "date",
            Self::Time => "time",
            Self::PeopleCount => "people_count",
            Self::ReservationReference => "reservation_reference",
            Self::MenuItem => "menu_item",
            Self::Allergen => "allergen",
            Self::DietaryRequirement => "dietary_requirement",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

/// Requirement definition for one workflow slot.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotDefinition {
    pub name: SlotName,
    pub slot_type: SlotType,
    pub required: bool,
    pub entity_types: Vec<EntityType>,
    pub prompt: I18nKey,
}

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
    pub slot: SlotName,
    pub message: String,
}

/// Container for filled slots with validation.
#[derive(Debug, Clone, Default)]
pub struct SlotBag {
    slots: HashMap<SlotName, SlotValue>,
}

impl SlotBag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fill a slot. Returns Err if value type does not match or validation fails.
    pub fn fill(
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
        self.validate(name, &value)?;
        self.slots.insert(name, value);
        Ok(())
    }

    pub fn get(&self, name: SlotName) -> Option<&SlotValue> {
        self.slots.get(&name)
    }

    pub fn is_filled(&self, name: SlotName) -> bool {
        self.slots.contains_key(&name)
    }

    pub fn filled_count(&self) -> usize {
        self.slots.len()
    }

    /// Domain-specific validation rules.
    fn validate(&self, name: SlotName, value: &SlotValue) -> Result<(), SlotError> {
        match (name, value) {
            (SlotName::Name, SlotValue::Text(s)) if s.trim().is_empty() || s.len() > 100 => {
                Err(SlotError {
                    slot: name,
                    message: "Name must be non-empty (max 100 chars)".to_string(),
                })
            }
            (SlotName::People, SlotValue::Number(n)) if *n < 1 || *n > 20 => Err(SlotError {
                slot: name,
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
            bag.fill(
                SlotName::Name,
                SlotType::Text,
                SlotValue::Text("Alice".into())
            )
            .is_ok()
        );
        assert!(bag.is_filled(SlotName::Name));
    }

    #[test]
    fn reject_wrong_type() {
        let mut bag = SlotBag::new();
        assert!(
            bag.fill(SlotName::Name, SlotType::Text, SlotValue::Number(42))
                .is_err()
        );
    }

    #[test]
    fn reject_empty_name() {
        let mut bag = SlotBag::new();
        assert!(
            bag.fill(SlotName::Name, SlotType::Text, SlotValue::Text("".into()))
                .is_err()
        );
    }

    #[test]
    fn reject_people_out_of_range() {
        let mut bag = SlotBag::new();
        assert!(
            bag.fill(SlotName::People, SlotType::Number, SlotValue::Number(0))
                .is_err()
        );
        assert!(
            bag.fill(SlotName::People, SlotType::Number, SlotValue::Number(21))
                .is_err()
        );
    }

    #[test]
    fn accept_people_in_range() {
        let mut bag = SlotBag::new();
        assert!(
            bag.fill(SlotName::People, SlotType::Number, SlotValue::Number(4))
                .is_ok()
        );
    }

    #[test]
    fn known_entity_label_maps_to_typed_variant() {
        assert_eq!(EntityType::new("menu_item"), EntityType::MenuItem);
    }

    #[test]
    fn unknown_entity_label_is_preserved() {
        assert_eq!(
            EntityType::new("dish_name"),
            EntityType::Unknown("dish_name".to_string())
        );
    }
}
