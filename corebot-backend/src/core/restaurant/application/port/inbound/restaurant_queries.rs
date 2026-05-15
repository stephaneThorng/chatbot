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
