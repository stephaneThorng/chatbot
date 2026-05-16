use uuid::Uuid;

use crate::core::restaurant::domain::model::{
    Reservation, ReservationDraft, RestaurantRepositoryError,
};

#[async_trait::async_trait]
pub trait RestaurantReservationRepositoryPort {
    async fn next_reference_index(
        &self,
        business_id: Uuid,
    ) -> Result<i64, RestaurantRepositoryError>;
    async fn create_reservation(
        &self,
        business_id: Uuid,
        reservation: ReservationDraft,
    ) -> Result<Reservation, RestaurantRepositoryError>;
    async fn find_by_reference(
        &self,
        business_id: Uuid,
        reference: &str,
    ) -> Result<Option<Reservation>, RestaurantRepositoryError>;
    async fn find_by_name(
        &self,
        business_id: Uuid,
        name: &str,
    ) -> Result<Vec<Reservation>, RestaurantRepositoryError>;
    async fn cancel_by_reference(
        &self,
        business_id: Uuid,
        reference: &str,
    ) -> Result<Option<Reservation>, RestaurantRepositoryError>;
}
