use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    MenuDietaryQuery, MenuItemDetailsQuery, MenuItemDetailsResult, MenuQuery, MenuSearchResult,
    PriceQuery,
};

#[async_trait::async_trait]
pub trait RestaurantMenuUseCase {
    async fn find_menu(&self, query: MenuQuery) -> MenuSearchResult;
    async fn find_menu_dietary(&self, query: MenuDietaryQuery) -> MenuSearchResult;
    async fn find_menu_item_details(&self, query: MenuItemDetailsQuery) -> MenuItemDetailsResult;
    async fn find_price(&self, query: PriceQuery) -> MenuSearchResult;
}
