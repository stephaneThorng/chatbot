use chrono::{NaiveDate, NaiveTime};
use sqlx::FromRow;

#[derive(FromRow)]
pub(crate) struct ReservationRow {
    pub reference: String,
    pub customer_name: String,
    pub reservation_date: NaiveDate,
    pub reservation_time: NaiveTime,
    pub people_count: i32,
}
