#[async_trait::async_trait]
pub trait RestaurantAccessibilityGatewayPort: Send + Sync {
    async fn get_accessibility_info(&self) -> String;
}
