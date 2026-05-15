use super::localized_datetime_formatter::{format_date, format_time};
use super::workflow_slot_reader::ReservationCreateSlots;

pub fn confirmation_summary(slots: &ReservationCreateSlots, lang: &str) -> String {
    rust_i18n::t!(
        "workflow.reservation_create.confirmation.prompt",
        locale = lang,
        name = slots.name,
        date = slots
            .date
            .map(|date| format_date(date, lang))
            .unwrap_or_default(),
        time = slots
            .time
            .map(|time| format_time(time, lang))
            .unwrap_or_default(),
        people = slots.people_count
    )
    .to_string()
}

pub fn completion_summary(slots: &ReservationCreateSlots, reference: &str, lang: &str) -> String {
    rust_i18n::t!(
        "workflow.reservation_create.completion.success",
        locale = lang,
        name = slots.name,
        date = slots
            .date
            .map(|date| format_date(date, lang))
            .unwrap_or_default(),
        time = slots
            .time
            .map(|time| format_time(time, lang))
            .unwrap_or_default(),
        people = slots.people_count,
        reference = reference
    )
    .to_string()
}
