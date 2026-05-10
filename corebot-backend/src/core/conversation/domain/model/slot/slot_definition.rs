use crate::core::conversation::domain::model::intent::I18nKey;

use super::{EntityType, SlotName, SlotType};

/// Requirement definition for one workflow slot.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotDefinition {
    pub name: SlotName,
    pub slot_type: SlotType,
    pub required: bool,
    pub entity_types: Vec<EntityType>,
    pub prompt: I18nKey,
}
