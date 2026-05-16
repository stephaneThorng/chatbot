use crate::core::restaurant::application::database_restaurant_service::DatabaseRestaurantService;
use crate::core::restaurant::application::port::inbound::restaurant_availability_usecase::RestaurantAvailabilityUseCase;
use crate::core::restaurant::application::port::inbound::restaurant_queries::RestaurantInfoResult;
use crate::core::restaurant::application::port::outbound::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;

use super::business_info_response_formatter::format_opening_hours;

#[async_trait::async_trait]
impl<B, M, R, A> RestaurantAvailabilityUseCase for DatabaseRestaurantService<B, M, R, A>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
    M: RestaurantMenuRepositoryPort + Send + Sync,
    R: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
{
    async fn get_opening_hours(&self) -> RestaurantInfoResult {
        let Ok(hours) = self
            .business_info_repository
            .opening_hours(self.business_id)
            .await
        else {
            return RestaurantInfoResult::new("hours_unavailable:");
        };
        RestaurantInfoResult::new(format_opening_hours(&hours))
    }
}
