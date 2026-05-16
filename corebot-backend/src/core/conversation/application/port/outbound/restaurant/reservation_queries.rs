#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReservationLookupQuery {
    pub reference: Option<String>,
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReservationCreateQuery {
    pub name: String,
    pub date: chrono::NaiveDate,
    pub time: chrono::NaiveTime,
    pub people_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReservationCancelQuery {
    pub reference: String,
    pub name: Option<String>,
    pub date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReservationFailure {
    RestaurantClosed,
    NoAvailability { next_slot: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReservationCancelFailure {
    NotFound,
}
