use chrono::{NaiveDate, NaiveTime};
use text2num::{Language, text2digits};

use crate::core::conversation::domain::service::date_resolver::{
    DateResolveError, resolve_date, resolve_time,
};

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
    Date(NaiveDate),
    Time(NaiveTime),
    Number(u32),
    Boolean(bool),
}

impl SlotDataValue {
    pub fn from_text(slot_type: SlotDataType, raw: &str, lang: &str) -> Option<Self> {
        let normalized = raw.trim();
        match slot_type {
            SlotDataType::Text => Some(Self::Text(normalized.to_string())),
            SlotDataType::Date => match resolve_date(normalized) {
                Ok(value) | Err(DateResolveError::PastDate(value)) => Some(Self::Date(value)),
                Err(DateResolveError::Unparseable) => None,
            },
            SlotDataType::Time => resolve_time(normalized).ok().map(Self::Time),
            SlotDataType::Number => parse_number_from_text(normalized, lang).map(Self::Number),
            SlotDataType::Boolean => None,
        }
    }

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

fn parse_number_from_text(raw: &str, lang: &str) -> Option<u32> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return None;
    }

    if let Ok(number) = normalized.parse::<u32>() {
        return Some(number);
    }

    let language = text2num_language(lang)?;
    let digits = text2digits(normalized, &language).ok()?;
    digits.parse::<u32>().ok()
}

fn text2num_language(lang: &str) -> Option<Language> {
    match lang.split(['-', '_']).next().unwrap_or(lang) {
        "en" => Some(Language::english()),
        "fr" => Some(Language::french()),
        "es" => Some(Language::spanish()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_from_text_accepts_digits() {
        assert_eq!(
            SlotDataValue::from_text(SlotDataType::Number, "21", "en"),
            Some(SlotDataValue::Number(21))
        );
        assert_eq!(
            SlotDataValue::from_text(SlotDataType::Number, "21", "id"),
            Some(SlotDataValue::Number(21))
        );
    }

    #[test]
    fn number_from_text_accepts_number_words() {
        assert_eq!(
            SlotDataValue::from_text(SlotDataType::Number, "twenty one", "en"),
            Some(SlotDataValue::Number(21))
        );
        assert_eq!(
            SlotDataValue::from_text(SlotDataType::Number, "one hundred", "en"),
            Some(SlotDataValue::Number(100))
        );
    }

    #[test]
    fn number_from_text_rejects_non_numbers() {
        assert_eq!(
            SlotDataValue::from_text(SlotDataType::Number, "many", "en"),
            None
        );
        assert_eq!(
            SlotDataValue::from_text(SlotDataType::Number, "a few", "en"),
            None
        );
    }
}
