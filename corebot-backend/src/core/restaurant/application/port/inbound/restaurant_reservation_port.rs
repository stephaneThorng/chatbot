use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    ReservationCancelQuery, ReservationCancelledResult, ReservationCreateQuery,
    ReservationCreatedResult, ReservationLookupQuery, ReservationLookupResult,
};
use crate::core::restaurant::domain::model::{ReservationCancelError, ReservationError};

#[async_trait::async_trait]
pub trait RestaurantReservationUseCase {
    async fn create_reservation(
        &self,
        query: ReservationCreateQuery,
    ) -> Result<ReservationCreatedResult, ReservationError>;
    async fn cancel_reservation(
        &self,
        query: ReservationCancelQuery,
    ) -> Result<ReservationCancelledResult, ReservationCancelError>;
    async fn check_reservation(&self, query: ReservationLookupQuery) -> ReservationLookupResult;
}
