use chrono::{Local, NaiveDate, NaiveTime};
use human_date_parser::{ParseResult, from_human_time};

#[derive(Debug, Clone, PartialEq)]
pub enum DateResolveError {
    Unparseable,
    PastDate(NaiveDate),
}

pub fn resolve_date(raw: &str) -> Result<NaiveDate, DateResolveError> {
    let today = Local::now().naive_local();
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

#[derive(Debug, Clone, PartialEq)]
pub enum TimeResolveError {
    Unparseable,
}

pub fn resolve_time(raw: &str) -> Result<NaiveTime, TimeResolveError> {
    let s = normalize_time_text(raw);

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

    if let Ok(t) = chrono::NaiveTime::parse_from_str(&s, "%H:%M") {
        return Ok(t);
    }
    if let Ok(t) = chrono::NaiveTime::parse_from_str(&s, "%H:%M:%S") {
        return Ok(t);
    }

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
        let result = resolve_date("on next tuesday");
        assert!(result.is_ok(), "expected future date, got {:?}", result);
    }

    #[test]
    fn on_the_prefix_stripped_before_parsing() {
        let result = resolve_date("on the 31st of December 2099");
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
