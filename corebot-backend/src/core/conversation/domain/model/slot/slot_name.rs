use std::fmt;

use crate::core::conversation::domain::model::slot::slot_value::SlotDataType;

/// Compile-time slot names used by workflows.
///
/// Each variant implicitly carries its data type and the NLU entity types
/// that can fill it — eliminating the need for separate `slot_type` and
/// `entity_types` fields in `SlotConfig`.
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
}

impl SlotName {
    /// The runtime data type this slot holds.
    pub fn data_type(self) -> SlotDataType {
        match self {
            SlotName::Name
            | SlotName::Reference
            | SlotName::MenuItem
            | SlotName::Allergen
            | SlotName::DietaryRequirement => SlotDataType::Text,
            SlotName::Date => SlotDataType::Date,
            SlotName::Time => SlotDataType::Time,
            SlotName::People => SlotDataType::Number,
        }
    }

    /// NLU entity labels (strings) that can fill this slot.
    pub fn entity_type_labels(self) -> &'static [&'static str] {
        match self {
            SlotName::Name => &["person"],
            SlotName::Date => &["date"],
            SlotName::Time => &["time"],
            SlotName::People => &["people_count"],
            SlotName::Reference => &["reservation_reference"],
            SlotName::MenuItem => &["menu_item"],
            SlotName::Allergen => &["allergen"],
            SlotName::DietaryRequirement => &["dietary_requirement"],
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Date => "date",
            Self::Time => "time",
            Self::People => "people",
            Self::Reference => "reference",
            Self::MenuItem => "menu_item",
            Self::Allergen => "allergen",
            Self::DietaryRequirement => "dietary_requirement",
        }
    }
}

impl fmt::Display for SlotName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
