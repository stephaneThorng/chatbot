#[async_trait::async_trait]
pub trait RestaurantContactGatewayPort: Send + Sync {
    async fn get_contact(&self) -> String;
}
