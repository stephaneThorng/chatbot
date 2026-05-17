use sqlx::PgPool;
use uuid::Uuid;

use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
use crate::core::conversation::domain::restaurant::model::{
    Reservation, ReservationDraft, RestaurantRepositoryError,
};

use super::models::ReservationRow;
use super::query_helpers::{repository_error, reservation_from_row};

#[derive(Clone)]
pub struct PostgresReservationRepository {
    pool: PgPool,
}

impl PostgresReservationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RestaurantReservationRepositoryPort for PostgresReservationRepository {
    async fn next_reference_index(
        &self,
        business_id: Uuid,
    ) -> Result<i64, RestaurantRepositoryError> {
        let count: (i64,) =
            sqlx::query_as("select count(*) + 1 from reservations where business_id = $1")
                .bind(business_id)
                .fetch_one(&self.pool)
                .await
                .map_err(repository_error)?;
        Ok(count.0)
    }

    async fn create_reservation(
        &self,
        business_id: Uuid,
        reservation: ReservationDraft,
    ) -> Result<Reservation, RestaurantRepositoryError> {
        sqlx::query_as::<_, ReservationRow>(
            r#"
            insert into reservations (
                id, business_id, reference, customer_name, reservation_date,
                reservation_time, people_count, status, created_at, updated_at
            )
            values ($1, $2, $3, $4, $5, $6, $7, 'confirmed', now(), now())
            returning reference, customer_name, reservation_date, reservation_time, people_count
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(business_id)
        .bind(reservation.reference)
        .bind(reservation.name)
        .bind(reservation.date)
        .bind(reservation.time)
        .bind(reservation.people_count as i32)
        .fetch_one(&self.pool)
        .await
        .map(reservation_from_row)
        .map_err(repository_error)
    }

    async fn find_by_reference(
        &self,
        business_id: Uuid,
        reference: &str,
    ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
        sqlx::query_as::<_, ReservationRow>(
            r#"
            select reference, customer_name, reservation_date, reservation_time, people_count
            from reservations
            where business_id = $1 and lower(reference) = lower($2) and status = 'confirmed'
            limit 1
            "#,
        )
        .bind(business_id)
        .bind(reference)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(reservation_from_row))
        .map_err(repository_error)
    }

    async fn find_by_name(
        &self,
        business_id: Uuid,
        name: &str,
    ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
        sqlx::query_as::<_, ReservationRow>(
            r#"
            select reference, customer_name, reservation_date, reservation_time, people_count
            from reservations
            where business_id = $1 and lower(customer_name) = lower($2) and status = 'confirmed'
            order by reservation_date, reservation_time
            "#,
        )
        .bind(business_id)
        .bind(name)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(reservation_from_row).collect())
        .map_err(repository_error)
    }

    async fn cancel_by_reference(
        &self,
        business_id: Uuid,
        reference: &str,
    ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
        sqlx::query_as::<_, ReservationRow>(
            r#"
            update reservations
            set status = 'cancelled', updated_at = now()
            where business_id = $1 and lower(reference) = lower($2) and status = 'confirmed'
            returning reference, customer_name, reservation_date, reservation_time, people_count
            "#,
        )
        .bind(business_id)
        .bind(reference)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(reservation_from_row))
        .map_err(repository_error)
    }
}
