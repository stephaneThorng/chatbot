/// Inbound port of the restaurant domain.
/// The restaurant adapter implements this trait to expose domain data.
pub trait RestaurantPort {
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
