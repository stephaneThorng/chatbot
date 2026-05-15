use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    EventQuery, FacilityQuery, LocationQuery, MenuDietaryQuery, MenuItemDetailsQuery, MenuQuery,
    PaymentMethodQuery, PriceQuery,
};

pub trait RestaurantInformationUseCase {
    fn get_opening_hours(&self) -> String;
    fn find_menu(&self, query: MenuQuery) -> String;
    fn find_menu_dietary(&self, query: MenuDietaryQuery) -> String;
    fn find_menu_item_details(&self, query: MenuItemDetailsQuery) -> String;
    fn find_location(&self, query: LocationQuery) -> String;
    fn get_contact(&self) -> String;
    fn find_payment_methods(&self, query: PaymentMethodQuery) -> String;
    fn find_price(&self, query: PriceQuery) -> String;
    fn get_takeaway_info(&self) -> String;
    fn find_event_info(&self, query: EventQuery) -> String;
    fn find_facility_info(&self, query: FacilityQuery) -> String;
    fn get_accessibility_info(&self) -> String;
    fn get_entertainment_info(&self) -> String;
}
