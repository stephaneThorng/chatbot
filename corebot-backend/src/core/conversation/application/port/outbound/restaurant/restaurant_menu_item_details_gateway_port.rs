use crate::core::conversation::application::port::outbound::restaurant::menu_queries::MenuItemDetailsQuery;

#[async_trait::async_trait]
pub trait RestaurantMenuItemDetailsGatewayPort: Send + Sync {
    async fn find_menu_item_details(&self, query: MenuItemDetailsQuery) -> String;
}
