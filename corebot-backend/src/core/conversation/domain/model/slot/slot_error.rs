use super::SlotName;

/// Error when filling a slot.
#[derive(Debug, PartialEq)]
pub struct SlotError {
    pub slot: SlotName,
    pub message: String,
}
