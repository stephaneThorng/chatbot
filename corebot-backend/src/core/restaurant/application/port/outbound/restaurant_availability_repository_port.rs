use chrono::{NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::core::restaurant::domain::model::{
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
