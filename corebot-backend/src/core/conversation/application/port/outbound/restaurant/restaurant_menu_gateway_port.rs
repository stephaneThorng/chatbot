use crate::core::conversation::application::port::outbound::restaurant::menu_queries::MenuQuery;

#[async_trait::async_trait]
pub trait RestaurantMenuGatewayPort: Send + Sync {
    async fn find_menu(&self, query: MenuQuery) -> String;
}
