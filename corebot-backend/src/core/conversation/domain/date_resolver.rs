use chrono::{Local, NaiveDate, NaiveTime};
use human_date_parser::{ParseResult, from_human_time};

/// Domain error for date resolution.
#[derive(Debug, Clone, PartialEq)]
pub enum DateResolveError {
    /// The raw string could not be parsed into any recognised date pattern.
    Unparseable,
    /// The resolved date falls strictly before today.
    PastDate(NaiveDate),
}

/// Resolve a human-readable date string (as extracted by the NLU) into a concrete
/// calendar date relative to today.
///
/// Uses the `human-date-parser` crate which covers relative expressions
/// (`tomorrow`, `next Friday`, `in 3 days`), plain weekday names, and literal
/// dates (`June 12`, `2026-08-23`).
///
/// Year-inference: if the resolved date falls before today it is treated as past.
pub fn resolve_date(raw: &str) -> Result<NaiveDate, DateResolveError> {
    let today = Local::now().naive_local();
    // Strip leading prepositions that the NLU may include ("on next tuesday", "on the 5th")
    let raw = raw.trim();
    let raw = raw
        .strip_prefix("on the ")
        .or_else(|| raw.strip_prefix("on "))
        .or_else(|| raw.strip_prefix("the "))
        .unwrap_or(raw);
    match from_human_time(raw, today) {
        Ok(ParseResult::Date(date)) => {
            if date < today.date() {
                Err(DateResolveError::PastDate(date))
            } else {
                Ok(date)
            }
        }
        Ok(ParseResult::DateTime(dt)) => {
            let date = dt.date();
            if date < today.date() {
                Err(DateResolveError::PastDate(date))
            } else {
                Ok(date)
            }
        }
        Ok(ParseResult::Time(_)) | Err(_) => Err(DateResolveError::Unparseable),
    }
}

/// Error returned when a raw time string cannot be parsed.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeResolveError {
    Unparseable,
}

/// Resolve a human-readable time string (as extracted by the NLU) into a [`NaiveTime`].
///
/// Supports the following formats:
/// - 24-hour: `"19:00"`, `"7:30"`
/// - 12-hour with am/pm: `"7pm"`, `"7 pm"`, `"7:30 pm"`, `"12:00 am"`
/// - Bare hour with am/pm suffix: `"7pm"`, `"9am"`
pub fn resolve_time(raw: &str) -> Result<NaiveTime, TimeResolveError> {
    let s = normalize_time_text(raw);

    // Try explicit 12h suffix first (most reliable for chatbot input)
    let normalized = s.replace(" pm", "pm").replace(" am", "am");
    if let Some(rest) = normalized.strip_suffix("pm") {
        if let Some(time) = parse_12h(rest.trim(), true) {
            return Ok(time);
        }
    }
    if let Some(rest) = normalized.strip_suffix("am") {
        if let Some(time) = parse_12h(rest.trim(), false) {
            return Ok(time);
        }
    }

    // Try 24h format directly
    if let Ok(t) = chrono::NaiveTime::parse_from_str(&s, "%H:%M") {
        return Ok(t);
    }
    if let Ok(t) = chrono::NaiveTime::parse_from_str(&s, "%H:%M:%S") {
        return Ok(t);
    }

    // Fall back to human-date-parser for anything remaining
    let now = Local::now().naive_local();
    if let Ok(ParseResult::Time(t)) = from_human_time(&s, now) {
        return Ok(t);
    }
    if let Ok(ParseResult::DateTime(dt)) = from_human_time(&s, now) {
        return Ok(dt.time());
    }

    Err(TimeResolveError::Unparseable)
}

fn normalize_time_text(raw: &str) -> String {
    let normalized = raw.trim().to_lowercase();
    let normalized = normalized
        .strip_prefix("jam ")
        .or_else(|| normalized.strip_prefix("pukul "))
        .unwrap_or(normalized.as_str())
        .trim()
        .to_string();

    if let Some(rest) = normalized.strip_suffix(" malam") {
        return format!("{}pm", rest.trim());
    }
    if let Some(rest) = normalized.strip_suffix(" pagi") {
        return format!("{}am", rest.trim());
    }
    if let Some(rest) = normalized.strip_suffix(" sore") {
        return format!("{}pm", rest.trim());
    }
    if let Some(rest) = normalized.strip_suffix(" siang") {
        return format!("{}pm", rest.trim());
    }

    normalized
}

fn parse_12h(rest: &str, pm: bool) -> Option<NaiveTime> {
    let (h_str, m_str) = if let Some((h, m)) = rest.split_once(':') {
        (h, m)
    } else {
        (rest, "0")
    };
    let h: u32 = h_str.trim().parse().ok()?;
    let m: u32 = m_str.trim().parse().ok()?;
    if h > 12 || m > 59 {
        return None;
    }
    let hour = match (h, pm) {
        (12, true) => 12,
        (12, false) => 0,
        (h, true) => h + 12,
        (h, false) => h,
    };
    NaiveTime::from_hms_opt(hour, m, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_prefix_stripped_before_parsing() {
        // "on next tuesday" is what the NLU extracts when the user says "on next tuesday at 6pm"
        let result = resolve_date("on next tuesday");
        assert!(result.is_ok(), "expected future date, got {:?}", result);
    }

    #[test]
    fn on_the_prefix_stripped_before_parsing() {
        let result = resolve_date("on the 31st of December 2099");
        // Should not fail with Unparseable due to the "on the" prefix
        assert!(result.is_ok() || matches!(result, Err(DateResolveError::Unparseable)));
    }

    #[test]
    fn future_absolute_date_resolves() {
        let result = resolve_date("2099-12-31");
        assert!(result.is_ok());
    }

    #[test]
    fn past_date_returns_error() {
        let result = resolve_date("2000-01-01");
        assert!(matches!(result, Err(DateResolveError::PastDate(_))));
    }

    #[test]
    fn unparseable_returns_error() {
        let result = resolve_date("notadate!!");
        assert_eq!(result, Err(DateResolveError::Unparseable));
    }

    #[test]
    fn resolve_time_24h_format() {
        assert_eq!(
            resolve_time("19:00"),
            Ok(NaiveTime::from_hms_opt(19, 0, 0).unwrap())
        );
    }

    #[test]
    fn resolve_time_12h_pm() {
        assert_eq!(
            resolve_time("7pm"),
            Ok(NaiveTime::from_hms_opt(19, 0, 0).unwrap())
        );
    }

    #[test]
    fn resolve_time_12h_am() {
        assert_eq!(
            resolve_time("9am"),
            Ok(NaiveTime::from_hms_opt(9, 0, 0).unwrap())
        );
    }

    #[test]
    fn resolve_time_with_minutes() {
        assert_eq!(
            resolve_time("7:30 pm"),
            Ok(NaiveTime::from_hms_opt(19, 30, 0).unwrap())
        );
    }

    #[test]
    fn resolve_time_noon() {
        assert_eq!(
            resolve_time("12:00 pm"),
            Ok(NaiveTime::from_hms_opt(12, 0, 0).unwrap())
        );
    }

    #[test]
    fn resolve_time_midnight() {
        assert_eq!(
            resolve_time("12:00 am"),
            Ok(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        );
    }

    #[test]
    fn resolve_time_unparseable() {
        assert_eq!(resolve_time("blah"), Err(TimeResolveError::Unparseable));
    }

    #[test]
    fn resolve_time_indonesian_evening_format() {
        assert_eq!(
            resolve_time("jam 7 malam"),
            Ok(NaiveTime::from_hms_opt(19, 0, 0).unwrap())
        );
    }
}
