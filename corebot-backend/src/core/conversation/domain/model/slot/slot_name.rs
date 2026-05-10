use std::fmt;

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
