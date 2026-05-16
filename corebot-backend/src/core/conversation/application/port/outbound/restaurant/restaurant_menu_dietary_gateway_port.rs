use crate::core::conversation::application::port::outbound::restaurant::menu_queries::MenuDietaryQuery;

#[async_trait::async_trait]
pub trait RestaurantMenuDietaryGatewayPort: Send + Sync {
    async fn find_menu_dietary(&self, query: MenuDietaryQuery) -> String;
}
