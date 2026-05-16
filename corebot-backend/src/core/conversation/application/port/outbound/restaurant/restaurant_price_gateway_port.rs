use crate::core::conversation::application::port::outbound::restaurant::menu_queries::PriceQuery;

#[async_trait::async_trait]
pub trait RestaurantPriceGatewayPort: Send + Sync {
    async fn find_price(&self, query: PriceQuery) -> String;
}
