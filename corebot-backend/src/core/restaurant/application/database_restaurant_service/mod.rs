mod availability_policy;
mod business_info_response_formatter;
mod menu_response_formatter;
mod query_mapper;
mod reservation_response_formatter;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use uuid::Uuid;

use crate::core::restaurant::application::port::inbound::restaurant_information_port::RestaurantInformationUseCase;
use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    EventQuery, FacilityQuery, FacilityResult, LocationQuery, MenuDietaryQuery,
    MenuItemDetailsQuery, MenuItemDetailsResult, MenuQuery, MenuSearchResult, PaymentMethodQuery,
    PaymentMethodsResult, PriceQuery, ReservationCancelQuery, ReservationCancelledResult,
    ReservationCreateQuery, ReservationCreatedResult, ReservationLookupQuery,
    ReservationLookupResult, RestaurantInfoResult,
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

use self::availability_policy::{can_seat, is_open_at};
use self::business_info_response_formatter::{
    facility_matches, fact_payload, format_opening_hours,
};
use self::menu_response_formatter::{contains_text, find_item, format_items, price_euros};
use self::query_mapper::map_price_filter;
use self::reservation_response_formatter::format_reservation_found;

pub struct DatabaseRestaurantService<B, M, R, A> {
    business_id: Uuid,
    locale: String,
    business_info_repository: B,
    menu_repository: M,
    reservation_repository: R,
    availability_repository: A,
}

impl<B, M, R, A> DatabaseRestaurantService<B, M, R, A> {
    pub fn new(
        business_id: Uuid,
        locale: impl Into<String>,
        business_info_repository: B,
        menu_repository: M,
        reservation_repository: R,
        availability_repository: A,
    ) -> Self {
        Self {
            business_id,
            locale: locale.into(),
            business_info_repository,
            menu_repository,
            reservation_repository,
            availability_repository,
        }
    }
}

#[async_trait::async_trait]
impl<B, M, R, A> RestaurantInformationUseCase for DatabaseRestaurantService<B, M, R, A>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
    M: RestaurantMenuRepositoryPort + Send + Sync,
    R: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
{
    async fn get_opening_hours(&self) -> RestaurantInfoResult {
        let Ok(hours) = self
            .business_info_repository
            .opening_hours(self.business_id)
            .await
        else {
            return RestaurantInfoResult::new("hours_unavailable:");
        };
        RestaurantInfoResult::new(format_opening_hours(&hours))
    }

    async fn find_menu(&self, query: MenuQuery) -> MenuSearchResult {
        if let Some(filter) = query.price_filter {
            let Ok(items) = self
                .menu_repository
                .menu_items_by_price(self.business_id, &self.locale, &map_price_filter(filter))
                .await
            else {
                return MenuSearchResult::new("no_results:");
            };
            if items.is_empty() {
                return MenuSearchResult::new("no_results:");
            }
            return MenuSearchResult::new(format!("price_results:{}", format_items(&items)));
        }

        let Ok(items) = self
            .menu_repository
            .menu_items(self.business_id, &self.locale)
            .await
        else {
            return MenuSearchResult::new("full_menu:");
        };

        if let Some(item_name) = query.price_item {
            if let Some(item) = find_item(&items, &item_name) {
                return MenuSearchResult::new(format!(
                    "item_found:{}|{}|{}",
                    item.name,
                    price_euros(item.price_cents),
                    item.allergens.join(",")
                ));
            }
            return MenuSearchResult::new("item_not_found:");
        }

        MenuSearchResult::new(format!("full_menu:{}", format_items(&items)))
    }

    async fn find_menu_dietary(&self, query: MenuDietaryQuery) -> MenuSearchResult {
        let Ok(items) = self
            .menu_repository
            .menu_items(self.business_id, &self.locale)
            .await
        else {
            return MenuSearchResult::new("dietary_no_filter:");
        };

        if let Some(requirement) = query.dietary_requirement {
            let req = requirement.to_lowercase();
            let matches = items
                .iter()
                .filter(|item| {
                    item.dietary
                        .iter()
                        .any(|value| value.to_lowercase().contains(&req))
                })
                .map(|item| item.name.clone())
                .collect::<Vec<_>>();
            if matches.is_empty() {
                return MenuSearchResult::new(format!("no_dietary:{requirement}"));
            }
            return MenuSearchResult::new(format!(
                "dietary_results:{}|{}",
                requirement,
                matches.join(", ")
            ));
        }

        let mut dietary = items
            .iter()
            .flat_map(|item| item.dietary.clone())
            .collect::<Vec<_>>();
        dietary.sort();
        dietary.dedup();
        MenuSearchResult::new(format!("dietary_no_filter:{}", dietary.join(", ")))
    }

    async fn find_menu_item_details(&self, query: MenuItemDetailsQuery) -> MenuItemDetailsResult {
        let Ok(items) = self
            .menu_repository
            .menu_items(self.business_id, &self.locale)
            .await
        else {
            return MenuItemDetailsResult::new("details_no_filter:");
        };

        match (query.menu_item, query.allergen) {
            (Some(item_name), Some(allergen)) => {
                if let Some(item) = find_item(&items, &item_name) {
                    if contains_text(&item.allergens, &allergen) {
                        return MenuItemDetailsResult::new(format!(
                            "contains:{item_name}|{allergen}"
                        ));
                    }
                    return MenuItemDetailsResult::new(format!(
                        "not_contains:{item_name}|{allergen}"
                    ));
                }
                MenuItemDetailsResult::new(format!("item_unknown:{item_name}"))
            }
            (Some(item_name), None) => {
                if let Some(item) = find_item(&items, &item_name) {
                    return MenuItemDetailsResult::new(format!(
                        "item_details:{}|{}|{}|{}",
                        item.name,
                        price_euros(item.price_cents),
                        if item.dietary.is_empty() {
                            "none".to_string()
                        } else {
                            item.dietary.join(", ")
                        },
                        if item.allergens.is_empty() {
                            "none".to_string()
                        } else {
                            item.allergens.join(", ")
                        }
                    ));
                }
                MenuItemDetailsResult::new(format!("item_unknown:{item_name}"))
            }
            (None, Some(allergen)) => {
                let matches = items
                    .iter()
                    .filter(|item| contains_text(&item.allergens, &allergen))
                    .map(|item| item.name.clone())
                    .collect::<Vec<_>>();
                if matches.is_empty() {
                    return MenuItemDetailsResult::new(format!("no_allergen_match:{allergen}"));
                }
                MenuItemDetailsResult::new(format!(
                    "allergen_found:{}|{}",
                    allergen,
                    matches.join(", ")
                ))
            }
            (None, None) => MenuItemDetailsResult::new("details_no_filter:"),
        }
    }

    async fn find_location(&self, query: LocationQuery) -> RestaurantInfoResult {
        let Ok(location) = self
            .business_info_repository
            .location(self.business_id)
            .await
        else {
            return RestaurantInfoResult::new("address:");
        };
        let Some(location) = location else {
            return RestaurantInfoResult::new("address:");
        };
        let address = match location.nearby_description {
            Some(nearby) if !nearby.is_empty() => format!("{} - {}", location.address_line, nearby),
            _ => location.address_line,
        };
        if let Some(near) = query.near {
            let normalized = near.to_lowercase();
            if address.to_lowercase().contains(&normalized) {
                return RestaurantInfoResult::new(format!("near_confirmed:{near}|{address}"));
            }
            return RestaurantInfoResult::new(format!("near_denied:{near}|{address}"));
        }
        RestaurantInfoResult::new(format!("address:{address}"))
    }

    async fn get_contact(&self) -> RestaurantInfoResult {
        let Ok(channels) = self
            .business_info_repository
            .contact_channels(self.business_id)
            .await
        else {
            return RestaurantInfoResult::new("contact:|");
        };
        let phone = channels
            .iter()
            .find(|channel| channel.channel_type == "phone")
            .map(|channel| channel.value.as_str())
            .unwrap_or("");
        let email = channels
            .iter()
            .find(|channel| channel.channel_type == "email")
            .map(|channel| channel.value.as_str())
            .unwrap_or("");
        RestaurantInfoResult::new(format!("contact:{phone}|{email}"))
    }

    async fn find_payment_methods(&self, query: PaymentMethodQuery) -> PaymentMethodsResult {
        let Ok(methods) = self
            .business_info_repository
            .payment_methods(self.business_id)
            .await
        else {
            return PaymentMethodsResult::new("all_methods:");
        };
        let all = methods
            .iter()
            .map(|method| method.method_code.clone())
            .collect::<Vec<_>>()
            .join(", ");
        if let Some(method) = query.method {
            if methods.iter().any(|candidate| {
                candidate
                    .method_code
                    .to_lowercase()
                    .contains(&method.to_lowercase())
            }) {
                return PaymentMethodsResult::new(format!("method_accepted:{method}|{all}"));
            }
            return PaymentMethodsResult::new(format!("method_not_accepted:{method}|{all}"));
        }
        PaymentMethodsResult::new(format!("all_methods:{all}"))
    }

    async fn find_price(&self, query: PriceQuery) -> MenuSearchResult {
        self.find_menu(MenuQuery {
            price_item: query.item,
            price_filter: query.price_filter,
        })
        .await
    }

    async fn get_takeaway_info(&self) -> RestaurantInfoResult {
        fact_payload(
            &self.business_info_repository,
            self.business_id,
            &self.locale,
            "takeaway",
        )
        .await
    }

    async fn find_event_info(&self, query: EventQuery) -> RestaurantInfoResult {
        let Ok(spaces) = self
            .business_info_repository
            .event_spaces(self.business_id)
            .await
        else {
            return RestaurantInfoResult::new("event_info:");
        };
        if let Some(location) = query.location {
            if let Some(space) = spaces
                .iter()
                .find(|space| location.to_lowercase().contains(&space.name.to_lowercase()))
            {
                let info = space
                    .contact
                    .clone()
                    .or_else(|| space.description.clone())
                    .unwrap_or_default();
                return RestaurantInfoResult::new(format!(
                    "event_space_available:{location}|{info}"
                ));
            }
            return RestaurantInfoResult::new(format!(
                "event_space_unavailable:{}|We have {} available for events.",
                location,
                spaces
                    .iter()
                    .map(|space| space.name.clone())
                    .collect::<Vec<_>>()
                    .join(" and ")
            ));
        }
        RestaurantInfoResult::new(format!(
            "event_info:{}",
            spaces
                .iter()
                .map(|space| {
                    let description = space.description.clone().unwrap_or_default();
                    format!("{} {}", space.name, description).trim().to_string()
                })
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    async fn find_facility_info(&self, query: FacilityQuery) -> FacilityResult {
        let Ok(facilities) = self
            .business_info_repository
            .facilities(self.business_id)
            .await
        else {
            return FacilityResult::new("all_facilities:");
        };
        if let Some(facility) = query.facility {
            if facilities
                .iter()
                .any(|candidate| facility_matches(&candidate.label, &facility))
            {
                return FacilityResult::new(format!("facility_available:{facility}"));
            }
            return FacilityResult::new(format!("facility_unavailable:{facility}"));
        }
        FacilityResult::new(format!(
            "all_facilities:{}",
            facilities
                .iter()
                .map(|facility| facility.label.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    async fn get_accessibility_info(&self) -> RestaurantInfoResult {
        fact_payload(
            &self.business_info_repository,
            self.business_id,
            &self.locale,
            "accessibility",
        )
        .await
    }

    async fn get_entertainment_info(&self) -> RestaurantInfoResult {
        fact_payload(
            &self.business_info_repository,
            self.business_id,
            &self.locale,
            "entertainment",
        )
        .await
    }
}

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
    use chrono::Weekday;
    use std::sync::{Arc, Mutex};

    use crate::core::restaurant::domain::model::{
        BusinessFact, BusinessLocation, ContactChannel, EventSpace, Facility, MenuItem,
        MenuPriceFilter, PaymentMethod, Reservation, RestaurantRepositoryError,
    };

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
