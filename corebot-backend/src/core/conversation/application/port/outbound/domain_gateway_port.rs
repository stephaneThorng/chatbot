use std::sync::Arc;

/// Outbound port - what the conversation use case needs from a domain.
/// Each domain (restaurant, hotel) provides its own implementation.
pub trait DomainGatewayPort {
    fn get_opening_hours(&self) -> String;
    fn get_menu(&self, price_item: Option<&str>, price_comparator: Option<&str>, price_amount: Option<&str>) -> String;
    fn get_menu_dietary(&self, dietary: Option<&str>) -> String;
    fn get_menu_item_details(&self, menu_item: Option<&str>, allergen: Option<&str>) -> String;
    fn get_location(&self, near: Option<&str>) -> String;
    fn get_contact(&self) -> String;
    fn get_payment_methods(&self, method: Option<&str>) -> String;
    fn get_price(&self, item: Option<&str>, price_comparator: Option<&str>, price_amount: Option<&str>) -> String;
    fn get_takeaway_info(&self) -> String;
    fn get_event_info(&self, location: Option<&str>) -> String;
    fn get_facility_info(&self, facility: Option<&str>) -> String;
    fn get_accessibility_info(&self) -> String;
    fn get_entertainment_info(&self) -> String;
    fn check_reservation(&self, reference: Option<&str>) -> String;
}

/// Blanket implementation so `Arc<D>` can be used wherever `D: DomainGatewayPort`.
impl<D: DomainGatewayPort + Send + Sync> DomainGatewayPort for Arc<D> {
    fn get_opening_hours(&self) -> String { (**self).get_opening_hours() }
    fn get_menu(&self, a: Option<&str>, b: Option<&str>, c: Option<&str>) -> String { (**self).get_menu(a, b, c) }
    fn get_menu_dietary(&self, a: Option<&str>) -> String { (**self).get_menu_dietary(a) }
    fn get_menu_item_details(&self, a: Option<&str>, b: Option<&str>) -> String { (**self).get_menu_item_details(a, b) }
    fn get_location(&self, a: Option<&str>) -> String { (**self).get_location(a) }
    fn get_contact(&self) -> String { (**self).get_contact() }
    fn get_payment_methods(&self, a: Option<&str>) -> String { (**self).get_payment_methods(a) }
    fn get_price(&self, a: Option<&str>, b: Option<&str>, c: Option<&str>) -> String { (**self).get_price(a, b, c) }
    fn get_takeaway_info(&self) -> String { (**self).get_takeaway_info() }
    fn get_event_info(&self, a: Option<&str>) -> String { (**self).get_event_info(a) }
    fn get_facility_info(&self, a: Option<&str>) -> String { (**self).get_facility_info(a) }
    fn get_accessibility_info(&self) -> String { (**self).get_accessibility_info() }
    fn get_entertainment_info(&self) -> String { (**self).get_entertainment_info() }
    fn check_reservation(&self, a: Option<&str>) -> String { (**self).check_reservation(a) }
}
