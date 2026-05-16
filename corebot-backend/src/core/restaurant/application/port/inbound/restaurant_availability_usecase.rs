use crate::core::restaurant::application::port::inbound::restaurant_queries::RestaurantInfoResult;

#[async_trait::async_trait]
pub trait RestaurantAvailabilityUseCase {
    async fn get_opening_hours(&self) -> RestaurantInfoResult;
}
