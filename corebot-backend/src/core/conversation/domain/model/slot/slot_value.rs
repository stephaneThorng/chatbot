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
