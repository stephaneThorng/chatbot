#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PriceFilter {
    pub comparator: String,
    pub amount: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuQuery {
    pub price_item: Option<String>,
    pub price_filter: Option<PriceFilter>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuDietaryQuery {
    pub dietary_requirement: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuItemDetailsQuery {
    pub menu_item: Option<String>,
    pub allergen: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocationQuery {
    pub near: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaymentMethodQuery {
    pub method: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PriceQuery {
    pub item: Option<String>,
    pub price_filter: Option<PriceFilter>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventQuery {
    pub location: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FacilityQuery {
    pub facility: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestaurantInfoResult {
    pub payload: String,
}

impl RestaurantInfoResult {
    pub fn new(payload: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuSearchResult {
    pub payload: String,
}

impl MenuSearchResult {
    pub fn new(payload: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItemDetailsResult {
    pub payload: String,
}

impl MenuItemDetailsResult {
    pub fn new(payload: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentMethodsResult {
    pub payload: String,
}

impl PaymentMethodsResult {
    pub fn new(payload: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FacilityResult {
    pub payload: String,
}

impl FacilityResult {
    pub fn new(payload: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
        }
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReservationCreatedResult {
    pub reference: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReservationCancelledResult {
    pub reference: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReservationLookupResult {
    pub payload: String,
}

impl ReservationLookupResult {
    pub fn new(payload: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
        }
    }
}
