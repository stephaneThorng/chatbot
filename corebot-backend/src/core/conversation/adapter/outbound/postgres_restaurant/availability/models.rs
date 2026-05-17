use chrono::{NaiveDate, NaiveTime};
use sqlx::FromRow;

#[derive(FromRow)]
pub(crate) struct ReservationSettingsRow {
    pub slot_minutes: i32,
    pub max_lookup_days: i32,
}

#[derive(FromRow)]
pub(crate) struct TableTypeRow {
    pub capacity: i32,
    pub table_count: i32,
}

#[derive(FromRow)]
pub(crate) struct OpeningHoursRow {
    pub day_of_week: i16,
    pub opens_at: NaiveTime,
    pub closes_at: NaiveTime,
    pub is_closed: bool,
}

#[derive(FromRow)]
pub(crate) struct ReservationRow {
    pub reference: String,
    pub customer_name: String,
    pub reservation_date: NaiveDate,
    pub reservation_time: NaiveTime,
    pub people_count: i32,
}
