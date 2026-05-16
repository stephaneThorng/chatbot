#[async_trait::async_trait]
pub trait RestaurantEntertainmentGatewayPort: Send + Sync {
    async fn get_entertainment_info(&self) -> String;
}
