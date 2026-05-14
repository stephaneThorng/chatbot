use crate::core::conversation::domain::model::intent::I18nKey;

use super::{EntityType, SlotName, SlotType};

/// Declarative constraint placed on a slot value at policy definition time.
///
/// Constraints are evaluated centrally by the workflow engine after each slot
/// fill, before `post_process` is ever reached.
#[derive(Debug, Clone, PartialEq)]
pub enum SlotConstraint {
    /// Text value must not exceed the given byte length.
    TextMaxLen(usize),
    /// Text value must be a valid email address (contains `@` with non-empty
    /// local and domain parts).
    EmailFormat,
    /// Numeric value must fall within the inclusive range `[min, max]`.
    NumberRange(u32, u32),
    /// Date value must resolve to a future (or today) calendar date.
    /// Uses [`crate::core::conversation::domain::date_resolver::resolve_date`].
    FutureDate,
}

impl SlotConstraint {
    /// Fallback i18n key used when the policy does not supply a custom error key.
    pub fn default_error_key(&self) -> &'static str {
        match self {
            SlotConstraint::TextMaxLen(_) => "system.constraint.text_max_len.error",
            SlotConstraint::EmailFormat => "system.constraint.email_format.error",
            SlotConstraint::NumberRange(_, _) => "system.constraint.number_range.error",
            SlotConstraint::FutureDate => "system.constraint.future_date.error",
        }
    }
}

/// A constraint entry on a slot: the rule plus an optional per-slot i18n
/// error key override.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotConstraintEntry {
    pub constraint: SlotConstraint,
    /// When `Some`, used in place of `constraint.default_error_key()`.
    pub error_key: Option<I18nKey>,
}

impl SlotConstraintEntry {
    pub fn new(constraint: SlotConstraint) -> Self {
        Self {
            constraint,
            error_key: None,
        }
    }

    pub fn with_error_key(constraint: SlotConstraint, key: &str) -> Self {
        Self {
            constraint,
            error_key: Some(I18nKey::new(key)),
        }
    }

    pub fn resolved_error_key(&self) -> &str {
        self.error_key
            .as_ref()
            .map(|k| k.0.as_str())
            .unwrap_or_else(|| self.constraint.default_error_key())
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
    /// Declarative constraints evaluated before `post_process`.
    pub constraints: Vec<SlotConstraintEntry>,
}
