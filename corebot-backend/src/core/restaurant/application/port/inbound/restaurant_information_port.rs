use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    EventQuery, FacilityQuery, FacilityResult, LocationQuery, MenuDietaryQuery,
    MenuItemDetailsQuery, MenuItemDetailsResult, MenuQuery, MenuSearchResult, PaymentMethodQuery,
    PaymentMethodsResult, PriceQuery, RestaurantInfoResult,
};

#[async_trait::async_trait]
pub trait RestaurantInformationUseCase {
    async fn get_opening_hours(&self) -> RestaurantInfoResult;
    async fn find_menu(&self, query: MenuQuery) -> MenuSearchResult;
    async fn find_menu_dietary(&self, query: MenuDietaryQuery) -> MenuSearchResult;
    async fn find_menu_item_details(&self, query: MenuItemDetailsQuery) -> MenuItemDetailsResult;
    async fn find_location(&self, query: LocationQuery) -> RestaurantInfoResult;
    async fn get_contact(&self) -> RestaurantInfoResult;
    async fn find_payment_methods(&self, query: PaymentMethodQuery) -> PaymentMethodsResult;
    async fn find_price(&self, query: PriceQuery) -> MenuSearchResult;
    async fn get_takeaway_info(&self) -> RestaurantInfoResult;
    async fn find_event_info(&self, query: EventQuery) -> RestaurantInfoResult;
    async fn find_facility_info(&self, query: FacilityQuery) -> FacilityResult;
    async fn get_accessibility_info(&self) -> RestaurantInfoResult;
    async fn get_entertainment_info(&self) -> RestaurantInfoResult;
}
