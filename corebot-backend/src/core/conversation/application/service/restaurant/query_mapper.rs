use crate::core::conversation::application::port::outbound::restaurant::menu_queries::PriceFilter;
use crate::core::conversation::domain::restaurant::model::MenuPriceFilter;

pub(super) fn map_price_filter(filter: PriceFilter) -> MenuPriceFilter {
    MenuPriceFilter {
        comparator: filter.comparator,
        amount: filter.amount,
    }
}
