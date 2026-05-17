use chrono::{Duration, NaiveDate, NaiveTime};
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::conversation::application::port::outbound::restaurant::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use crate::core::conversation::domain::restaurant::model::{
    OpeningHours, Reservation, ReservationSettings, RestaurantRepositoryError, TableType,
};

use super::models::{OpeningHoursRow, ReservationRow, ReservationSettingsRow, TableTypeRow};
use super::query_helpers::{repository_error, reservation_from_row, weekday_from_database};

#[derive(Clone)]
pub struct PostgresAvailabilityRepository {
    pool: PgPool,
}

impl PostgresAvailabilityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RestaurantAvailabilityRepositoryPort for PostgresAvailabilityRepository {
    async fn reservation_settings(
        &self,
        business_id: Uuid,
    ) -> Result<ReservationSettings, RestaurantRepositoryError> {
        sqlx::query_as::<_, ReservationSettingsRow>(
            "select slot_minutes, max_lookup_days from restaurant_reservation_settings where business_id = $1",
        )
        .bind(business_id)
        .fetch_one(&self.pool)
        .await
        .map(|row| ReservationSettings {
            slot_minutes: row.slot_minutes.max(1) as u32,
            max_lookup_days: row.max_lookup_days.max(1) as u32,
        })
        .map_err(repository_error)
    }

    async fn table_types(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<TableType>, RestaurantRepositoryError> {
        sqlx::query_as::<_, TableTypeRow>(
            "select capacity, table_count from restaurant_table_types where business_id = $1 order by capacity",
        )
        .bind(business_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| TableType {
                    capacity: row.capacity.max(0) as u32,
                    count: row.table_count.max(0) as u32,
                })
                .collect()
        })
        .map_err(repository_error)
    }

    async fn opening_hours(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
        sqlx::query_as::<_, OpeningHoursRow>(
            "select day_of_week, opens_at, closes_at, is_closed from restaurant_opening_hours where business_id = $1 order by day_of_week",
        )
        .bind(business_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| OpeningHours {
                    day_of_week: weekday_from_database(row.day_of_week),
                    opens_at: row.opens_at,
                    closes_at: row.closes_at,
                    is_closed: row.is_closed,
                })
                .collect()
        })
        .map_err(repository_error)
    }

    async fn is_closed_at(
        &self,
        business_id: Uuid,
        date: NaiveDate,
        time: NaiveTime,
        slot_minutes: u32,
    ) -> Result<bool, RestaurantRepositoryError> {
        let start = date.and_time(time);
        let end = start + Duration::minutes(slot_minutes as i64);
        let exists: (bool,) = sqlx::query_as(
            r#"
            select exists(
                select 1
                from business_closures
                where business_id = $1
                  and starts_at < $3
                  and ends_at > $2
            )
            "#,
        )
        .bind(business_id)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await
        .map_err(repository_error)?;
        Ok(exists.0)
    }

    async fn reservations_near(
        &self,
        business_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
        sqlx::query_as::<_, ReservationRow>(
            r#"
            select reference, customer_name, reservation_date, reservation_time, people_count
            from reservations
            where business_id = $1 and reservation_date = $2 and status = 'confirmed'
            order by reservation_time
            "#,
        )
        .bind(business_id)
        .bind(date)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(reservation_from_row).collect())
        .map_err(repository_error)
    }
}
