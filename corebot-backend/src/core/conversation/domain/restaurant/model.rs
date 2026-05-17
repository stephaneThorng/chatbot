use std::collections::BTreeMap;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Weekday};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuPriceFilter {
    pub comparator: String,
    pub amount: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    pub name: String,
    pub ingredients: Vec<String>,
    pub dietary: Vec<String>,
    pub allergens: Vec<String>,
    pub price_cents: i32,
    pub currency: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReservationDraft {
    pub reference: String,
    pub name: String,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub people_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reservation {
    pub reference: String,
    pub name: String,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub people_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableType {
    pub capacity: u32,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReservationSettings {
    pub slot_minutes: u32,
    pub max_lookup_days: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpeningHours {
    pub day_of_week: Weekday,
    pub opens_at: NaiveTime,
    pub closes_at: NaiveTime,
    pub is_closed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BusinessLocation {
    pub address_line: String,
    pub nearby_description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContactChannel {
    pub channel_type: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentMethod {
    pub method_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facility {
    pub facility_code: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventSpace {
    pub name: String,
    pub description: Option<String>,
    pub contact: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BusinessFact {
    pub fact_type: String,
    pub title: Option<String>,
    pub content: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestaurantRepositoryError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReservationError {
    RestaurantClosed,
    NoAvailability { next_slot: Option<NaiveDateTime> },
    RepositoryUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReservationCancelError {
    NotFound,
    RepositoryUnavailable,
}
