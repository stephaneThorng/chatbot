use chrono::{Datelike, NaiveDate, Weekday};

use crate::core::conversation::domain::date_resolver::{DateResolveError, DateResolver};

/// Adapter implementation of [`DateResolver`] for English date expressions.
///
/// Covers every format produced by the `generate_dataset.py` EN value list:
/// - Relative : `today`, `tomorrow`, weekday names (`Friday`),
///              `next <weekday>` (`next Monday`, `next Tuesday`)
/// - Literal  : `June 12`, `on July 8`, `on August 23 2026`
/// - ISO      : `2026-08-23`
/// - European : `23/08/2026`
///
/// Year-inference rule (no explicit year):
///   - Compute the date in the current year.
///   - If that date < today → use year+1 (rolls forward).
///
/// A resolved date < today returns [`DateResolveError::PastDate`].
pub struct EnglishDateResolver;

impl DateResolver for EnglishDateResolver {
    fn resolve(&self, raw: &str, today: NaiveDate) -> Result<NaiveDate, DateResolveError> {
        let s = raw.trim().to_lowercase().replace(',', " ");
        let s = s.trim_start_matches("on ").trim();
        let s = strip_weekday_prefix(s).unwrap_or(s);

        let date = resolve_relative(s, today)
            .or_else(|| resolve_iso(s))
            .or_else(|| resolve_european(s))
            .or_else(|| resolve_literal_en(s, today))
            .ok_or(DateResolveError::Unparseable)?;

        if date < today {
            Err(DateResolveError::PastDate(date))
        } else {
            Ok(date)
        }
    }
}

fn strip_weekday_prefix(s: &str) -> Option<&str> {
    for weekday in [
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
    ] {
        if let Some(rest) = s.strip_prefix(weekday) {
            let rest = rest.trim();
            if !rest.is_empty() {
                return Some(rest);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Relative expressions
// ---------------------------------------------------------------------------

fn resolve_relative(s: &str, today: NaiveDate) -> Option<NaiveDate> {
    match s {
        "today" => Some(today),
        "tomorrow" => today.succ_opt(),
        _ => {
            // plain weekday ("friday") → next occurrence from tomorrow
            if let Some(wd) = parse_weekday(s) {
                return Some(next_weekday(today, wd, false));
            }
            // "next <weekday>"
            if let Some(rest) = s.strip_prefix("next ") {
                if let Some(wd) = parse_weekday(rest) {
                    return Some(next_weekday(today, wd, true));
                }
            }
            None
        }
    }
}

fn parse_weekday(s: &str) -> Option<Weekday> {
    match s {
        "monday" => Some(Weekday::Mon),
        "tuesday" => Some(Weekday::Tue),
        "wednesday" => Some(Weekday::Wed),
        "thursday" => Some(Weekday::Thu),
        "friday" => Some(Weekday::Fri),
        "saturday" => Some(Weekday::Sat),
        "sunday" => Some(Weekday::Sun),
        _ => None,
    }
}

/// Returns the next occurrence of `wd` after today (or next week if `force_next`).
fn next_weekday(today: NaiveDate, wd: Weekday, force_next: bool) -> NaiveDate {
    let today_wd = today.weekday();
    let days_ahead =
        (wd.num_days_from_monday() as i64 - today_wd.num_days_from_monday() as i64).rem_euclid(7);
    let days_ahead = if days_ahead == 0 || force_next {
        days_ahead + 7
    } else {
        days_ahead
    };
    today + chrono::Duration::days(days_ahead)
}

// ---------------------------------------------------------------------------
// ISO 8601 : 2026-08-23
// ---------------------------------------------------------------------------

fn resolve_iso(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

// ---------------------------------------------------------------------------
// European : 23/08/2026
// ---------------------------------------------------------------------------

fn resolve_european(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%d/%m/%Y").ok()
}

// ---------------------------------------------------------------------------
// Literal English : "june 12", "august 23 2026"
// ---------------------------------------------------------------------------

fn resolve_literal_en(s: &str, today: NaiveDate) -> Option<NaiveDate> {
    // Try "month day year"  e.g. "august 23 2026"
    if let Ok(d) = NaiveDate::parse_from_str(s, "%B %d %Y") {
        return Some(d);
    }
    // Try "month day" e.g. "june 12" — infer year
    if let Ok(d) = NaiveDate::parse_from_str(&format!("{} {}", s, today.year()), "%B %d %Y") {
        if d >= today {
            return Some(d);
        }
        // already passed this year → next year
        return NaiveDate::from_ymd_opt(today.year() + 1, d.month(), d.day());
    }
    // Try "day month" e.g. "8 july" (less common in EN but covers edge cases)
    if let Ok(d) = NaiveDate::parse_from_str(&format!("{} {}", s, today.year()), "%d %B %Y") {
        if d >= today {
            return Some(d);
        }
        return NaiveDate::from_ymd_opt(today.year() + 1, d.month(), d.day());
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, 11).unwrap()
    }

    fn resolve(raw: &str) -> Result<NaiveDate, DateResolveError> {
        EnglishDateResolver.resolve(raw, today())
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn today_resolves_to_current_date() {
        assert_eq!(resolve("today"), Ok(today()));
    }

    #[test]
    fn tomorrow_resolves_to_next_day() {
        assert_eq!(resolve("tomorrow"), Ok(date(2026, 5, 12)));
    }

    #[test]
    fn plain_weekday_rolls_to_next_occurrence() {
        // 2026-05-11 is a Monday; next Friday is 2026-05-15
        assert_eq!(resolve("Friday"), Ok(date(2026, 5, 15)));
    }

    #[test]
    fn next_weekday_skips_current_week() {
        // next Monday from 2026-05-11 (Monday) → 2026-05-18
        assert_eq!(resolve("next Monday"), Ok(date(2026, 5, 18)));
    }

    #[test]
    fn iso_format_parsed() {
        assert_eq!(resolve("2026-08-23"), Ok(date(2026, 8, 23)));
    }

    #[test]
    fn european_format_parsed() {
        assert_eq!(resolve("23/08/2026"), Ok(date(2026, 8, 23)));
    }

    #[test]
    fn literal_month_day_future_same_year() {
        assert_eq!(resolve("June 12"), Ok(date(2026, 6, 12)));
    }

    #[test]
    fn literal_month_day_past_rolls_to_next_year() {
        // January 5 is in the past relative to 2026-05-11
        assert_eq!(resolve("January 5"), Ok(date(2027, 1, 5)));
    }

    #[test]
    fn literal_with_explicit_year() {
        assert_eq!(resolve("on August 23 2026"), Ok(date(2026, 8, 23)));
    }

    #[test]
    fn on_prefix_stripped_before_iso() {
        assert_eq!(resolve("on July 8"), Ok(date(2026, 7, 8)));
    }

    #[test]
    fn weekday_day_month_parsed() {
        assert_eq!(resolve("on Friday 17 July"), Ok(date(2026, 7, 17)));
    }

    #[test]
    fn past_date_returns_past_date_error() {
        assert_eq!(
            resolve("2025-01-01"),
            Err(DateResolveError::PastDate(date(2025, 1, 1)))
        );
    }

    #[test]
    fn unparseable_returns_error() {
        assert!(matches!(
            resolve("blahblah"),
            Err(DateResolveError::Unparseable)
        ));
    }
}
