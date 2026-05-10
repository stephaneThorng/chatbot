use super::SlotName;

/// Error when filling a slot.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotError {
    pub slot: SlotName,
    pub message: String,
}
