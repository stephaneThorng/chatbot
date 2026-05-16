use crate::core::restaurant::application::port::inbound::restaurant_queries::PriceFilter;
use crate::core::restaurant::domain::model::MenuPriceFilter;

pub(super) fn map_price_filter(filter: PriceFilter) -> MenuPriceFilter {
    MenuPriceFilter {
        comparator: filter.comparator,
        amount: filter.amount,
    }
}
