use crate::core::conversation::application::port::outbound::restaurant::business_info_queries::LocationQuery;

#[async_trait::async_trait]
pub trait RestaurantLocationGatewayPort: Send + Sync {
    async fn find_location(&self, query: LocationQuery) -> String;
}
