use crate::core::conversation::application::port::outbound::restaurant::reservation_queries::{
    ReservationCancelFailure, ReservationCancelQuery, ReservationCreateQuery, ReservationFailure,
    ReservationLookupQuery,
};

#[async_trait::async_trait]
pub trait RestaurantReservationGatewayPort: Send + Sync {
    async fn create_reservation(
        &self,
        query: ReservationCreateQuery,
    ) -> Result<String, ReservationFailure>;

    async fn cancel_reservation(
        &self,
        query: ReservationCancelQuery,
    ) -> Result<String, ReservationCancelFailure>;

    async fn check_reservation(&self, query: ReservationLookupQuery) -> String;
}
