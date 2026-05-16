use crate::core::conversation::application::port::outbound::restaurant::business_info_queries::EventQuery;

#[async_trait::async_trait]
pub trait RestaurantEventGatewayPort: Send + Sync {
    async fn find_event_info(&self, query: EventQuery) -> String;
}
