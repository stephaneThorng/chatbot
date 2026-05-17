use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

use crate::core::conversation::application::port::outbound::restaurant::reservation_queries::{
    ReservationCancelFailure, ReservationCancelQuery, ReservationCreateQuery, ReservationFailure,
    ReservationLookupQuery,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
use crate::core::conversation::application::service::restaurant::ConversationRestaurantReservationService;
use crate::core::conversation::domain::restaurant::model::{OpeningHours, Reservation, ReservationDraft, ReservationSettings, TableType};
use uuid::Uuid;

use super::availability_policy::{can_seat, is_open_at};

impl<R, A> ConversationRestaurantReservationService<R, A>
where
    R: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
{
    pub async fn create_reservation(
        &self,
        business_id: Uuid,
        query: ReservationCreateQuery,
    ) -> Result<String, ReservationFailure> {
        let settings = self
            .availability_repository
            .reservation_settings(business_id)
            .await
            .map_err(|_| ReservationFailure::NoAvailability { next_slot: None })?;
        let opening_hours = self
            .availability_repository
            .opening_hours(business_id)
            .await
            .map_err(|_| ReservationFailure::NoAvailability { next_slot: None })?;
        let tables = self
            .availability_repository
            .table_types(business_id)
            .await
            .map_err(|_| ReservationFailure::NoAvailability { next_slot: None })?;
        let reservations = self
            .availability_repository
            .reservations_near(business_id, query.date)
            .await
            .map_err(|_| ReservationFailure::NoAvailability { next_slot: None })?;
        let closed = self
            .availability_repository
            .is_closed_at(business_id, query.date, query.time, settings.slot_minutes)
            .await
            .map_err(|_| ReservationFailure::NoAvailability { next_slot: None })?;

        if closed
            || !is_open_at(
                &opening_hours,
                query.date,
                query.time,
                settings.slot_minutes,
            )
        {
            return Err(ReservationFailure::RestaurantClosed);
        }
        if !can_seat(
            &tables,
            &reservations,
            query.date,
            query.time,
            query.people_count,
            settings.slot_minutes,
        ) {
            let next_slot = self
                .next_available_slot(
                    business_id,
                    &settings,
                    &opening_hours,
                    &tables,
                    query.date,
                    query.time,
                    query.people_count,
                )
                .await?;
            return Err(ReservationFailure::NoAvailability {
                next_slot: next_slot.map(|dt| dt.format("%Y-%m-%d %H:%M").to_string()),
            });
        }

        let reference_index = self
            .reservation_repository
            .next_reference_index(business_id)
            .await
            .map_err(|_| ReservationFailure::NoAvailability { next_slot: None })?;
        let reference = format!("REST-{reference_index:06X}");
        let reservation = self
            .reservation_repository
            .create_reservation(
                business_id,
                ReservationDraft {
                    reference,
                    name: query.name,
                    date: query.date,
                    time: query.time,
                    people_count: query.people_count,
                },
            )
            .await
            .map_err(|_| ReservationFailure::NoAvailability { next_slot: None })?;
        Ok(format!("created:{}", reservation.reference))
    }

    pub async fn cancel_reservation(
        &self,
        business_id: Uuid,
        query: ReservationCancelQuery,
    ) -> Result<String, ReservationCancelFailure> {
        let cancelled = self
            .reservation_repository
            .cancel_by_reference(business_id, &query.reference)
            .await
            .map_err(|_| ReservationCancelFailure::NotFound)?;

        let Some(cancelled) = cancelled else {
            return Err(ReservationCancelFailure::NotFound);
        };

        if query
            .name
            .as_ref()
            .is_some_and(|name| !cancelled.name.eq_ignore_ascii_case(name))
        {
            return Err(ReservationCancelFailure::NotFound);
        }

        if query.date.is_some_and(|date| cancelled.date != date) {
            return Err(ReservationCancelFailure::NotFound);
        }

        Ok(format!("cancelled:{}", cancelled.reference))
    }

    pub async fn check_reservation(
        &self,
        business_id: Uuid,
        query: ReservationLookupQuery,
    ) -> String {
        if let Some(reference) = query.reference {
            let Ok(reservation) = self
                .reservation_repository
                .find_by_reference(business_id, &reference)
                .await
            else {
                return format!("not_found:{reference}");
            };
            if let Some(reservation) = reservation {
                return format_reservation_found(&reservation);
            }
            return format!("not_found:{reference}");
        }

        if let Some(name) = query.name {
            let Ok(reservations) = self
                .reservation_repository
                .find_by_name(business_id, &name)
                .await
            else {
                return format!("name_not_found:{name}");
            };
            if reservations.is_empty() {
                return format!("name_not_found:{name}");
            }
            return format!(
                "listed:{}|{}",
                name,
                reservations
                    .iter()
                    .map(|reservation| format!(
                        "{}~{}~{}~{}",
                        reservation.reference,
                        reservation.date,
                        reservation.time.format("%H:%M"),
                        reservation.people_count
                    ))
                    .collect::<Vec<_>>()
                    .join(";")
            );
        }

        "no_reference_or_name:".to_string()
    }

    async fn next_available_slot(
        &self,
        business_id: Uuid,
        settings: &ReservationSettings,
        opening_hours: &[OpeningHours],
        tables: &[TableType],
        from_date: NaiveDate,
        from_time: NaiveTime,
        people: u32,
    ) -> Result<Option<NaiveDateTime>, ReservationFailure> {
        let slot_step = Duration::minutes(settings.slot_minutes as i64);
        let mut candidate = NaiveDateTime::new(from_date, from_time) + slot_step;

        for _ in 0..(settings.max_lookup_days * 24 * 60 / settings.slot_minutes) {
            let date = candidate.date();
            let time = candidate.time();
            let closed = self
                .availability_repository
                .is_closed_at(business_id, date, time, settings.slot_minutes)
                .await
                .map_err(|_| ReservationFailure::NoAvailability { next_slot: None })?;
            let reservations = self
                .availability_repository
                .reservations_near(business_id, date)
                .await
                .map_err(|_| ReservationFailure::NoAvailability { next_slot: None })?;

            if !closed
                && is_open_at(opening_hours, date, time, settings.slot_minutes)
                && can_seat(
                    tables,
                    &reservations,
                    date,
                    time,
                    people,
                    settings.slot_minutes,
                )
            {
                return Ok(Some(candidate));
            }
            candidate += slot_step;
        }
        Ok(None)
    }
}

pub(super) fn format_reservation_found(reservation: &Reservation) -> String {
    format!(
        "found:{}|{}|{}|{}|{}",
        reservation.reference,
        reservation.name,
        reservation.date,
        reservation.time.format("%H:%M"),
        reservation.people_count
    )
}
