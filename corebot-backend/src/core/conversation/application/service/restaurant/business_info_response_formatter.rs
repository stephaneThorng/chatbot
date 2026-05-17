use chrono::Timelike;

use crate::core::conversation::domain::restaurant::model::OpeningHours;

pub(crate) fn format_opening_hours(hours: &[OpeningHours]) -> String {
    if hours.is_empty() {
        return "hours_unavailable:".to_string();
    }
    let Some(first_open) = hours.iter().find(|entry| !entry.is_closed) else {
        return "Closed".to_string();
    };

    fn format_time(time: chrono::NaiveTime) -> String {
        let suffix = if time.hour() < 12 { "am" } else { "pm" };
        let hour = match time.hour() % 12 {
            0 => 12,
            value => value,
        };
        if time.minute() == 0 {
            format!("{hour}{suffix}")
        } else {
            format!("{hour}:{:02}{suffix}", time.minute())
        }
    }

    format!(
        "Mon-Sun {}-{}",
        format_time(first_open.opens_at),
        format_time(first_open.closes_at)
    )
}

pub(crate) fn facility_matches(candidate: &str, requested: &str) -> bool {
    fn normalize(value: &str) -> String {
        value
            .to_lowercase()
            .replace("seats", "seating")
            .replace("seat", "seating")
            .replace("air conditioning", "ac")
            .replace('-', " ")
    }
    let candidate = normalize(candidate);
    let requested = normalize(requested);
    candidate.contains(&requested)
        || requested.contains(&candidate)
        || (requested == "parking" && candidate.contains("parking"))
        || (requested == "ac" && candidate.contains("ac"))
}
