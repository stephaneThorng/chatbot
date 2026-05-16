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
pub struct PriceQuery {
    pub item: Option<String>,
    pub price_filter: Option<PriceFilter>,
}
