use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::restaurant::application::port::inbound::restaurant_trait::RestaurantPort;

/// Outbound adapter - bridges conversation and the restaurant domain.
/// Implements DomainGateway by delegating every call to RestaurantPort.
pub struct RestaurantDomainGateway<R: RestaurantPort> {
    restaurant: R,
}

impl<R: RestaurantPort> RestaurantDomainGateway<R> {
    pub fn new(restaurant: R) -> Self {
        Self { restaurant }
    }
}

impl<R: RestaurantPort> DomainGatewayPort for RestaurantDomainGateway<R> {
    fn get_opening_hours(&self) -> String { self.restaurant.get_opening_hours() }
    fn get_menu(&self, price_item: Option<&str>, price_comparator: Option<&str>, price_amount: Option<&str>) -> String { self.restaurant.get_menu(price_item, price_comparator, price_amount) }
    fn get_menu_dietary(&self, dietary: Option<&str>) -> String { self.restaurant.get_menu_dietary(dietary) }
    fn get_menu_item_details(&self, menu_item: Option<&str>, allergen: Option<&str>) -> String { self.restaurant.get_menu_item_details(menu_item, allergen) }
    fn get_location(&self, near: Option<&str>) -> String { self.restaurant.get_location(near) }
    fn get_contact(&self) -> String { self.restaurant.get_contact() }
    fn get_payment_methods(&self, method: Option<&str>) -> String { self.restaurant.get_payment_methods(method) }
    fn get_price(&self, item: Option<&str>, price_comparator: Option<&str>, price_amount: Option<&str>) -> String { self.restaurant.get_price(item, price_comparator, price_amount) }
    fn get_takeaway_info(&self) -> String { self.restaurant.get_takeaway_info() }
    fn get_event_info(&self, location: Option<&str>) -> String { self.restaurant.get_event_info(location) }
    fn get_facility_info(&self, facility: Option<&str>) -> String { self.restaurant.get_facility_info(facility) }
    fn get_accessibility_info(&self) -> String { self.restaurant.get_accessibility_info() }
    fn get_entertainment_info(&self) -> String { self.restaurant.get_entertainment_info() }
    fn check_reservation(&self, reference: Option<&str>) -> String { self.restaurant.check_reservation(reference) }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubRestaurantPort;

    impl RestaurantPort for StubRestaurantPort {
        fn get_opening_hours(&self) -> String { "Mon-Sun 9am-10pm".to_string() }
        fn get_menu(&self, _: Option<&str>, _: Option<&str>, _: Option<&str>) -> String { "full_menu:pizza".to_string() }
        fn get_menu_dietary(&self, _: Option<&str>) -> String { "dietary_no_filter:".to_string() }
        fn get_menu_item_details(&self, _: Option<&str>, _: Option<&str>) -> String { "details_no_filter:".to_string() }
        fn get_location(&self, _: Option<&str>) -> String { "address:123 Main St".to_string() }
        fn get_contact(&self) -> String { "contact:+33123456789|test@example.com".to_string() }
        fn get_payment_methods(&self, _: Option<&str>) -> String { "all_methods:cash".to_string() }
        fn get_price(&self, _: Option<&str>, _: Option<&str>, _: Option<&str>) -> String { "price_general:€10".to_string() }
        fn get_takeaway_info(&self) -> String { "takeaway:yes|Yes".to_string() }
        fn get_event_info(&self, _: Option<&str>) -> String { "event_info:Yes".to_string() }
        fn get_facility_info(&self, _: Option<&str>) -> String { "all_facilities:wifi".to_string() }
        fn get_accessibility_info(&self) -> String { "accessibility:yes|Yes".to_string() }
        fn get_entertainment_info(&self) -> String { "entertainment:yes|Live music".to_string() }
        fn check_reservation(&self, _: Option<&str>) -> String { "no_reference:".to_string() }
    }

    #[test]
    fn delegates_opening_hours_to_restaurant_port() {
        let gateway = RestaurantDomainGateway::new(StubRestaurantPort);
        assert_eq!(gateway.get_opening_hours(), "Mon-Sun 9am-10pm");
    }

    #[test]
    fn delegates_check_reservation_to_restaurant_port() {
        let gateway = RestaurantDomainGateway::new(StubRestaurantPort);
        assert_eq!(gateway.check_reservation(None), "no_reference:");
    }
}
