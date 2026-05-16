use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

use crate::core::restaurant::application::database_restaurant_service::DatabaseRestaurantService;
use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    ReservationCancelQuery, ReservationCancelledResult, ReservationCreateQuery,
    ReservationCreatedResult, ReservationLookupQuery, ReservationLookupResult,
};
use crate::core::restaurant::application::port::inbound::restaurant_reservation_port::RestaurantReservationUseCase;
use crate::core::restaurant::application::port::outbound::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
use crate::core::restaurant::domain::model::{
    OpeningHours, ReservationCancelError, ReservationDraft, ReservationError,
    ReservationSettings, TableType,
};

use super::availability_policy::{can_seat, is_open_at};
use super::reservation_response_formatter::format_reservation_found;

#[async_trait::async_trait]
impl<B, M, R, A> RestaurantReservationUseCase for DatabaseRestaurantService<B, M, R, A>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
    M: RestaurantMenuRepositoryPort + Send + Sync,
    R: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
{
    async fn create_reservation(
        &self,
        query: ReservationCreateQuery,
    ) -> Result<ReservationCreatedResult, ReservationError> {
        let settings = self
            .availability_repository
            .reservation_settings(self.business_id)
            .await
            .map_err(|_| ReservationError::RepositoryUnavailable)?;
        let opening_hours = self
            .availability_repository
            .opening_hours(self.business_id)
            .await
            .map_err(|_| ReservationError::RepositoryUnavailable)?;
        let tables = self
            .availability_repository
            .table_types(self.business_id)
            .await
            .map_err(|_| ReservationError::RepositoryUnavailable)?;
        let reservations = self
            .availability_repository
            .reservations_near(self.business_id, query.date)
            .await
            .map_err(|_| ReservationError::RepositoryUnavailable)?;
        let closed = self
            .availability_repository
            .is_closed_at(
                self.business_id,
                query.date,
                query.time,
                settings.slot_minutes,
            )
            .await
            .map_err(|_| ReservationError::RepositoryUnavailable)?;

        if closed
            || !is_open_at(
                &opening_hours,
                query.date,
                query.time,
                settings.slot_minutes,
            )
        {
            return Err(ReservationError::RestaurantClosed);
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
                    &settings,
                    &opening_hours,
                    &tables,
                    query.date,
                    query.time,
                    query.people_count,
                )
                .await?;
            return Err(ReservationError::NoAvailability { next_slot });
        }

        let reference_index = self
            .reservation_repository
            .next_reference_index(self.business_id)
            .await
            .map_err(|_| ReservationError::RepositoryUnavailable)?;
        let reference = format!("REST-{reference_index:06X}");
        let reservation = self
            .reservation_repository
            .create_reservation(
                self.business_id,
                ReservationDraft {
                    reference,
                    name: query.name,
                    date: query.date,
                    time: query.time,
                    people_count: query.people_count,
                },
            )
            .await
            .map_err(|_| ReservationError::RepositoryUnavailable)?;
        Ok(ReservationCreatedResult {
            reference: reservation.reference,
        })
    }

    async fn cancel_reservation(
        &self,
        query: ReservationCancelQuery,
    ) -> Result<ReservationCancelledResult, ReservationCancelError> {
        let cancelled = self
            .reservation_repository
            .cancel_by_reference(self.business_id, &query.reference)
            .await
            .map_err(|_| ReservationCancelError::RepositoryUnavailable)?;

        let Some(cancelled) = cancelled else {
            return Err(ReservationCancelError::NotFound);
        };

        if query
            .name
            .as_ref()
            .is_some_and(|name| !cancelled.name.eq_ignore_ascii_case(name))
        {
            return Err(ReservationCancelError::NotFound);
        }

        if query.date.is_some_and(|date| cancelled.date != date) {
            return Err(ReservationCancelError::NotFound);
        }

        Ok(ReservationCancelledResult {
            reference: cancelled.reference,
        })
    }

    async fn check_reservation(&self, query: ReservationLookupQuery) -> ReservationLookupResult {
        if let Some(reference) = query.reference {
            let Ok(reservation) = self
                .reservation_repository
                .find_by_reference(self.business_id, &reference)
                .await
            else {
                return ReservationLookupResult::new(format!("not_found:{reference}"));
            };
            if let Some(reservation) = reservation {
                return ReservationLookupResult::new(format_reservation_found(&reservation));
            }
            return ReservationLookupResult::new(format!("not_found:{reference}"));
        }

        if let Some(name) = query.name {
            let Ok(reservations) = self
                .reservation_repository
                .find_by_name(self.business_id, &name)
                .await
            else {
                return ReservationLookupResult::new(format!("name_not_found:{name}"));
            };
            if reservations.is_empty() {
                return ReservationLookupResult::new(format!("name_not_found:{name}"));
            }
            return ReservationLookupResult::new(format!(
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
            ));
        }

        ReservationLookupResult::new("no_reference_or_name:")
    }
}

impl<B, M, R, A> DatabaseRestaurantService<B, M, R, A>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
    M: RestaurantMenuRepositoryPort + Send + Sync,
    R: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
{
    async fn next_available_slot(
        &self,
        settings: &ReservationSettings,
        opening_hours: &[OpeningHours],
        tables: &[TableType],
        from_date: NaiveDate,
        from_time: NaiveTime,
        people: u32,
    ) -> Result<Option<NaiveDateTime>, ReservationError> {
        let slot_step = Duration::minutes(settings.slot_minutes as i64);
        let mut candidate = NaiveDateTime::new(from_date, from_time) + slot_step;

        for _ in 0..(settings.max_lookup_days * 24 * 60 / settings.slot_minutes) {
            let date = candidate.date();
            let time = candidate.time();
            let closed = self
                .availability_repository
                .is_closed_at(self.business_id, date, time, settings.slot_minutes)
                .await
                .map_err(|_| ReservationError::RepositoryUnavailable)?;
            let reservations = self
                .availability_repository
                .reservations_near(self.business_id, date)
                .await
                .map_err(|_| ReservationError::RepositoryUnavailable)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::restaurant::domain::model::{
        BusinessFact, BusinessLocation, ContactChannel, EventSpace, Facility, MenuItem,
        MenuPriceFilter, PaymentMethod, Reservation, RestaurantRepositoryError,
    };
    use chrono::Weekday;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    #[derive(Clone)]
    struct FakeRepository {
        reservations: Arc<Mutex<Vec<Reservation>>>,
        closed: bool,
    }

    impl FakeRepository {
        fn new() -> Self {
            Self {
                reservations: Arc::new(Mutex::new(vec![])),
                closed: false,
            }
        }

        fn with_closed_period() -> Self {
            Self {
                reservations: Arc::new(Mutex::new(vec![])),
                closed: true,
            }
        }
    }

    fn business_id() -> Uuid {
        Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()
    }

    fn service(
        repository: FakeRepository,
    ) -> DatabaseRestaurantService<FakeRepository, FakeRepository, FakeRepository, FakeRepository>
    {
        DatabaseRestaurantService::new(
            business_id(),
            "en",
            repository.clone(),
            repository.clone(),
            repository.clone(),
            repository,
        )
    }

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2099, 6, 12).unwrap()
    }

    fn time() -> NaiveTime {
        NaiveTime::from_hms_opt(19, 0, 0).unwrap()
    }

    #[async_trait::async_trait]
    impl RestaurantBusinessInfoRepositoryPort for FakeRepository {
        async fn opening_hours(
            &self,
            _: Uuid,
        ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
            Ok(opening_hours())
        }

        async fn location(
            &self,
            _: Uuid,
        ) -> Result<Option<BusinessLocation>, RestaurantRepositoryError> {
            Ok(None)
        }

        async fn contact_channels(
            &self,
            _: Uuid,
        ) -> Result<Vec<ContactChannel>, RestaurantRepositoryError> {
            Ok(vec![])
        }

        async fn payment_methods(
            &self,
            _: Uuid,
        ) -> Result<Vec<PaymentMethod>, RestaurantRepositoryError> {
            Ok(vec![])
        }

        async fn facilities(&self, _: Uuid) -> Result<Vec<Facility>, RestaurantRepositoryError> {
            Ok(vec![])
        }

        async fn facts(
            &self,
            _: Uuid,
            _: &str,
        ) -> Result<Vec<BusinessFact>, RestaurantRepositoryError> {
            Ok(vec![])
        }

        async fn event_spaces(
            &self,
            _: Uuid,
        ) -> Result<Vec<EventSpace>, RestaurantRepositoryError> {
            Ok(vec![])
        }
    }

    #[async_trait::async_trait]
    impl RestaurantMenuRepositoryPort for FakeRepository {
        async fn menu_items(
            &self,
            _: Uuid,
            _: &str,
        ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
            Ok(vec![])
        }

        async fn menu_items_by_price(
            &self,
            _: Uuid,
            _: &str,
            _: &MenuPriceFilter,
        ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
            Ok(vec![])
        }
    }

    #[async_trait::async_trait]
    impl RestaurantReservationRepositoryPort for FakeRepository {
        async fn next_reference_index(&self, _: Uuid) -> Result<i64, RestaurantRepositoryError> {
            Ok(self.reservations.lock().unwrap().len() as i64 + 1)
        }

        async fn create_reservation(
            &self,
            _: Uuid,
            reservation: ReservationDraft,
        ) -> Result<Reservation, RestaurantRepositoryError> {
            let reservation = Reservation {
                reference: reservation.reference,
                name: reservation.name,
                date: reservation.date,
                time: reservation.time,
                people_count: reservation.people_count,
            };
            self.reservations.lock().unwrap().push(reservation.clone());
            Ok(reservation)
        }

        async fn find_by_reference(
            &self,
            _: Uuid,
            reference: &str,
        ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
            Ok(self
                .reservations
                .lock()
                .unwrap()
                .iter()
                .find(|reservation| reservation.reference.eq_ignore_ascii_case(reference))
                .cloned())
        }

        async fn find_by_name(
            &self,
            _: Uuid,
            name: &str,
        ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
            Ok(self
                .reservations
                .lock()
                .unwrap()
                .iter()
                .filter(|reservation| reservation.name.eq_ignore_ascii_case(name))
                .cloned()
                .collect())
        }

        async fn cancel_by_reference(
            &self,
            _: Uuid,
            reference: &str,
        ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
            let mut reservations = self.reservations.lock().unwrap();
            let Some(index) = reservations
                .iter()
                .position(|reservation| reservation.reference.eq_ignore_ascii_case(reference))
            else {
                return Ok(None);
            };
            Ok(Some(reservations.remove(index)))
        }
    }

    #[async_trait::async_trait]
    impl RestaurantAvailabilityRepositoryPort for FakeRepository {
        async fn reservation_settings(
            &self,
            _: Uuid,
        ) -> Result<ReservationSettings, RestaurantRepositoryError> {
            Ok(ReservationSettings {
                slot_minutes: 120,
                max_lookup_days: 7,
            })
        }

        async fn table_types(&self, _: Uuid) -> Result<Vec<TableType>, RestaurantRepositoryError> {
            Ok(vec![TableType {
                capacity: 4,
                count: 1,
            }])
        }

        async fn opening_hours(
            &self,
            _: Uuid,
        ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
            Ok(opening_hours())
        }

        async fn is_closed_at(
            &self,
            _: Uuid,
            _: NaiveDate,
            _: NaiveTime,
            _: u32,
        ) -> Result<bool, RestaurantRepositoryError> {
            Ok(self.closed)
        }

        async fn reservations_near(
            &self,
            _: Uuid,
            _: NaiveDate,
        ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
            Ok(self.reservations.lock().unwrap().clone())
        }
    }

    fn opening_hours() -> Vec<OpeningHours> {
        [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ]
        .into_iter()
        .map(|day| OpeningHours {
            day_of_week: day,
            opens_at: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            closes_at: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            is_closed: false,
        })
        .collect()
    }

    #[tokio::test]
    async fn create_reservation_succeeds_with_repository_backed_service() {
        let result = service(FakeRepository::new())
            .create_reservation(ReservationCreateQuery {
                name: "Alice".to_string(),
                date: date(),
                time: time(),
                people_count: 2,
            })
            .await
            .unwrap();

        assert_eq!(result.reference, "REST-000001");
    }

    #[tokio::test]
    async fn create_reservation_fails_when_business_has_manual_closure() {
        let result = service(FakeRepository::with_closed_period())
            .create_reservation(ReservationCreateQuery {
                name: "Alice".to_string(),
                date: date(),
                time: time(),
                people_count: 2,
            })
            .await;

        assert_eq!(result, Err(ReservationError::RestaurantClosed));
    }

    #[tokio::test]
    async fn check_reservation_by_name_uses_repository_records() {
        let repository = FakeRepository::new();
        let service = service(repository);
        let _ = service
            .create_reservation(ReservationCreateQuery {
                name: "Alice".to_string(),
                date: date(),
                time: time(),
                people_count: 2,
            })
            .await
            .unwrap();

        let result = service
            .check_reservation(ReservationLookupQuery {
                reference: None,
                name: Some("Alice".to_string()),
            })
            .await;

        assert!(result.payload.starts_with("listed:Alice|REST-000001"));
    }

    #[tokio::test]
    async fn cancel_reservation_marks_existing_reference_as_cancelled() {
        let repository = FakeRepository::new();
        let service = service(repository);
        let created = service
            .create_reservation(ReservationCreateQuery {
                name: "Alice".to_string(),
                date: date(),
                time: time(),
                people_count: 2,
            })
            .await
            .unwrap();

        let cancelled = service
            .cancel_reservation(ReservationCancelQuery {
                reference: created.reference.clone(),
                name: Some("Alice".to_string()),
                date: Some(date()),
            })
            .await
            .unwrap();

        assert_eq!(cancelled.reference, created.reference);
        let lookup = service
            .check_reservation(ReservationLookupQuery {
                reference: Some(created.reference),
                name: None,
            })
            .await;
        assert!(lookup.payload.starts_with("not_found:"));
    }
}
