use std::sync::Arc;

use crate::core::conversation::application::port::outbound::restaurant_queries::{
    EventQuery, FacilityQuery, LocationQuery, MenuDietaryQuery, MenuItemDetailsQuery, MenuQuery,
    PaymentMethodQuery, PriceQuery,
};

pub trait RestaurantInformationPort {
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

impl<T: RestaurantInformationPort + Send + Sync> RestaurantInformationPort for Arc<T> {
    fn get_opening_hours(&self) -> String {
        self.as_ref().get_opening_hours()
    }

    fn find_menu(&self, query: MenuQuery) -> String {
        self.as_ref().find_menu(query)
    }

    fn find_menu_dietary(&self, query: MenuDietaryQuery) -> String {
        self.as_ref().find_menu_dietary(query)
    }

    fn find_menu_item_details(&self, query: MenuItemDetailsQuery) -> String {
        self.as_ref().find_menu_item_details(query)
    }

    fn find_location(&self, query: LocationQuery) -> String {
        self.as_ref().find_location(query)
    }

    fn get_contact(&self) -> String {
        self.as_ref().get_contact()
    }

    fn find_payment_methods(&self, query: PaymentMethodQuery) -> String {
        self.as_ref().find_payment_methods(query)
    }

    fn find_price(&self, query: PriceQuery) -> String {
        self.as_ref().find_price(query)
    }

    fn get_takeaway_info(&self) -> String {
        self.as_ref().get_takeaway_info()
    }

    fn find_event_info(&self, query: EventQuery) -> String {
        self.as_ref().find_event_info(query)
    }

    fn find_facility_info(&self, query: FacilityQuery) -> String {
        self.as_ref().find_facility_info(query)
    }

    fn get_accessibility_info(&self) -> String {
        self.as_ref().get_accessibility_info()
    }

    fn get_entertainment_info(&self) -> String {
        self.as_ref().get_entertainment_info()
    }
}
