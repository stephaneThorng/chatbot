use chrono::NaiveTime;
use serde_json::Value;
use sqlx::FromRow;

#[derive(FromRow)]
pub(crate) struct OpeningHoursRow {
    pub day_of_week: i16,
    pub opens_at: NaiveTime,
    pub closes_at: NaiveTime,
    pub is_closed: bool,
}

#[derive(FromRow)]
pub(crate) struct LocationRow {
    pub address_line: String,
    pub nearby_description: Option<String>,
}

#[derive(FromRow)]
pub(crate) struct ContactChannelRow {
    pub channel_type: String,
    pub value: String,
}

#[derive(FromRow)]
pub(crate) struct PaymentMethodRow {
    pub method_code: String,
}

#[derive(FromRow)]
pub(crate) struct FacilityRow {
    pub facility_code: String,
    pub label: String,
}

#[derive(FromRow)]
pub(crate) struct BusinessFactRow {
    pub fact_type: String,
    pub title: Option<String>,
    pub content: String,
    pub metadata: Value,
}

#[derive(FromRow)]
pub(crate) struct EventSpaceRow {
    pub name: String,
    pub description: Option<String>,
    pub contact: Option<String>,
}
