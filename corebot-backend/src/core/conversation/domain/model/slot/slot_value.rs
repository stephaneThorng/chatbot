/// Runtime type tag used for slot validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotDataType {
    Text,
    Date,
    Time,
    Number,
    Boolean,
}

/// Validated slot value.
#[derive(Debug, Clone, PartialEq)]
pub enum SlotDataValue {
    Text(String),
    Date(String),
    Time(String),
    Number(u32),
    Boolean(bool),
}

impl SlotDataValue {
    /// Check if this value matches the expected type.
    pub fn matches_type(&self, slot_type: SlotDataType) -> bool {
        matches!(
            (self, slot_type),
            (SlotDataValue::Text(_), SlotDataType::Text)
                | (SlotDataValue::Date(_), SlotDataType::Date)
                | (SlotDataValue::Time(_), SlotDataType::Time)
                | (SlotDataValue::Number(_), SlotDataType::Number)
                | (SlotDataValue::Boolean(_), SlotDataType::Boolean)
        )
    }
}
