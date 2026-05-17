use std::sync::Arc;
use uuid::Uuid;

use crate::core::conversation::domain::restaurant::model::{
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

#[async_trait::async_trait]
impl<T> RestaurantReservationRepositoryPort for Arc<T>
where
    T: RestaurantReservationRepositoryPort + Send + Sync + ?Sized,
{
    async fn next_reference_index(
        &self,
        business_id: Uuid,
    ) -> Result<i64, RestaurantRepositoryError> {
        self.as_ref().next_reference_index(business_id).await
    }

    async fn create_reservation(
        &self,
        business_id: Uuid,
        reservation: ReservationDraft,
    ) -> Result<Reservation, RestaurantRepositoryError> {
        self.as_ref()
            .create_reservation(business_id, reservation)
            .await
    }

    async fn find_by_reference(
        &self,
        business_id: Uuid,
        reference: &str,
    ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
        self.as_ref()
            .find_by_reference(business_id, reference)
            .await
    }

    async fn find_by_name(
        &self,
        business_id: Uuid,
        name: &str,
    ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
        self.as_ref().find_by_name(business_id, name).await
    }

    async fn cancel_by_reference(
        &self,
        business_id: Uuid,
        reference: &str,
    ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
        self.as_ref()
            .cancel_by_reference(business_id, reference)
            .await
    }
}
