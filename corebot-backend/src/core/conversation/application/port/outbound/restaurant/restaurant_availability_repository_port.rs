use chrono::{NaiveDate, NaiveTime};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::conversation::domain::restaurant::model::{
    OpeningHours, Reservation, ReservationSettings, RestaurantRepositoryError, TableType,
};

#[async_trait::async_trait]
pub trait RestaurantAvailabilityRepositoryPort {
    async fn reservation_settings(
        &self,
        business_id: Uuid,
    ) -> Result<ReservationSettings, RestaurantRepositoryError>;
    async fn table_types(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<TableType>, RestaurantRepositoryError>;
    async fn opening_hours(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError>;
    async fn is_closed_at(
        &self,
        business_id: Uuid,
        date: NaiveDate,
        time: NaiveTime,
        slot_minutes: u32,
    ) -> Result<bool, RestaurantRepositoryError>;
    async fn reservations_near(
        &self,
        business_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<Reservation>, RestaurantRepositoryError>;
}

#[async_trait::async_trait]
impl<T> RestaurantAvailabilityRepositoryPort for Arc<T>
where
    T: RestaurantAvailabilityRepositoryPort + Send + Sync + ?Sized,
{
    async fn reservation_settings(
        &self,
        business_id: Uuid,
    ) -> Result<ReservationSettings, RestaurantRepositoryError> {
        self.as_ref().reservation_settings(business_id).await
    }

    async fn table_types(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<TableType>, RestaurantRepositoryError> {
        self.as_ref().table_types(business_id).await
    }

    async fn opening_hours(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
        self.as_ref().opening_hours(business_id).await
    }

    async fn is_closed_at(
        &self,
        business_id: Uuid,
        date: NaiveDate,
        time: NaiveTime,
        slot_minutes: u32,
    ) -> Result<bool, RestaurantRepositoryError> {
        self.as_ref()
            .is_closed_at(business_id, date, time, slot_minutes)
            .await
    }

    async fn reservations_near(
        &self,
        business_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
        self.as_ref().reservations_near(business_id, date).await
    }
}
