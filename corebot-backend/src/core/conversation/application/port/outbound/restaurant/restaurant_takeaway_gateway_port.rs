#[async_trait::async_trait]
pub trait RestaurantTakeawayGatewayPort: Send + Sync {
    async fn get_takeaway_info(&self) -> String;
}
