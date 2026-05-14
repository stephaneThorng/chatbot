use crate::core::conversation::domain::model::intent::I18nKey;

use super::SlotName;

/// Declarative constraint placed on a slot value at config definition time.
///
/// Constraints are evaluated centrally by the workflow engine after each slot
/// fill, before `post_process` is ever reached.
#[derive(Debug, Clone, PartialEq)]
pub enum SlotConstraint {
    /// Text value must not exceed the given byte length.
    TextMaxLen(usize),
    /// Text value must be a valid email address.
    EmailFormat,
    /// Numeric value must fall within the inclusive range `[min, max]`.
    NumberRange(u32, u32),
    /// Date value must resolve to a future (or today) calendar date.
    FutureDate,
}

impl SlotConstraint {
    /// Fallback i18n key used when the config does not supply a custom error key.
    pub fn default_error_key(&self) -> &'static str {
        match self {
            SlotConstraint::TextMaxLen(_) => "system.constraint.text_max_len.error",
            SlotConstraint::EmailFormat => "system.constraint.email_format.error",
            SlotConstraint::NumberRange(_, _) => "system.constraint.number_range.error",
            SlotConstraint::FutureDate => "system.constraint.future_date.error",
        }
    }
}

/// A constraint entry: the rule plus an optional per-slot i18n error key override.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotConstraintEntry {
    pub constraint: SlotConstraint,
    pub error_key: Option<I18nKey>,
}

impl SlotConstraintEntry {
    pub fn new(constraint: SlotConstraint) -> Self {
        Self { constraint, error_key: None }
    }

    pub fn with_error_key(constraint: SlotConstraint, key: &str) -> Self {
        Self { constraint, error_key: Some(I18nKey::new(key)) }
    }

    pub fn resolved_error_key(&self) -> &str {
        self.error_key
            .as_ref()
            .map(|k| k.0.as_str())
            .unwrap_or_else(|| self.constraint.default_error_key())
    }
}

/// Declarative definition of one workflow slot.
///
/// `name` encodes the expected data type and NLU entity types — no separate
/// `slot_type` or `entity_types` fields are needed.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotConfig {
    pub name: SlotName,
    pub required: bool,
    pub prompt: I18nKey,
    pub constraints: Vec<SlotConstraintEntry>,
}
