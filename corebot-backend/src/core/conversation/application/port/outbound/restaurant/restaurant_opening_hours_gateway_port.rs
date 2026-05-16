#[async_trait::async_trait]
pub trait RestaurantOpeningHoursGatewayPort: Send + Sync {
    async fn get_opening_hours(&self) -> String;
}
