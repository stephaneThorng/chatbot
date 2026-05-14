use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use std::sync::Mutex;

use crate::core::restaurant::application::port::inbound::restaurant_information_port::RestaurantInformationUseCase;
use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    EventQuery, FacilityQuery, LocationQuery, MenuDietaryQuery, MenuItemDetailsQuery, MenuQuery,
    PaymentMethodQuery, PriceFilter, PriceQuery, ReservationCreateQuery, ReservationLookupQuery,
};
use crate::core::restaurant::application::port::inbound::restaurant_reservation_port::RestaurantReservationUseCase;
use crate::core::restaurant::domain::model::{
    MenuItem, Reservation, ReservationError, RestaurantConfig,
};

/// In-memory restaurant application service for v1.
/// Owns both static catalogue data and the live reservation list.
pub struct RestaurantService {
    config: RestaurantConfig,
    menu: Vec<MenuItem>,
    reservations: Mutex<Vec<Reservation>>,
    payment_methods: Vec<String>,
    facilities: Vec<String>,
}

impl RestaurantService {
    pub fn new() -> Self {
        Self {
            config: RestaurantConfig::default_v1(),
            menu: Self::seed_menu(),
            reservations: Mutex::new(Self::seed_reservations()),
            payment_methods: vec![
                "credit card".to_string(),
                "cash".to_string(),
                "Apple Pay".to_string(),
                "Google Pay".to_string(),
                "Visa".to_string(),
                "Mastercard".to_string(),
                "contactless".to_string(),
            ],
            facilities: vec![
                "baby seat".to_string(),
                "parking".to_string(),
                "wifi".to_string(),
                "high chairs".to_string(),
                "outdoor seating".to_string(),
                "private room".to_string(),
                "bike parking".to_string(),
            ],
        }
    }

    // -----------------------------------------------------------------------
    // Seeding
    // -----------------------------------------------------------------------

    fn seed_menu() -> Vec<MenuItem> {
        vec![
            MenuItem::new("pizza", &["vegetarian"], &["gluten", "dairy"], 12),
            MenuItem::new(
                "salad",
                &["vegan", "vegetarian", "gluten-free", "dairy-free", "nut-free"],
                &[],
                8,
            ),
            MenuItem::new("chocolate cake", &["vegetarian"], &["gluten", "dairy", "eggs"], 6),
            MenuItem::new("fried rice", &["gluten-free"], &["eggs", "soy"], 10),
            MenuItem::new("vegetarian pasta", &["vegetarian"], &["gluten", "dairy", "eggs"], 11),
            MenuItem::new("seafood soup", &["gluten-free", "dairy-free"], &["shellfish", "soy"], 14),
            MenuItem::new("beef burger", &[], &["gluten", "dairy", "eggs", "sesame"], 14),
            MenuItem::new(
                "chicken satay",
                &["halal", "gluten-free", "dairy-free"],
                &["peanuts", "soy"],
                13,
            ),
            MenuItem::new(
                "vegan curry",
                &["vegan", "vegetarian", "halal", "gluten-free", "dairy-free"],
                &["soy"],
                11,
            ),
            MenuItem::new("kids pasta", &["vegetarian"], &["gluten", "dairy", "eggs"], 8),
            MenuItem::new("set menu", &[], &["gluten", "dairy"], 35),
            MenuItem::new("lunch special", &[], &["gluten"], 15),
            MenuItem::new("kids menu", &["vegetarian"], &["gluten", "dairy"], 10),
            MenuItem::new("breakfast menu", &["vegetarian"], &["gluten", "dairy", "eggs"], 12),
            MenuItem::new("family menu", &[], &["gluten", "dairy"], 60),
            MenuItem::new("tasting menu", &[], &["gluten", "dairy", "shellfish"], 75),
            MenuItem::new("dessert menu", &["vegetarian"], &["gluten", "dairy", "eggs"], 18),
        ]
    }

    fn parse_seed_time(s: &str) -> NaiveTime {
        // Formats in seed data: "7:00 pm", "8:00 pm", "12:00 pm", "6:00 pm", etc.
        let s = s.trim().to_lowercase();
        let s = s.replace(" pm", "pm").replace(" am", "am");
        if let Some(rest) = s.strip_suffix("pm") {
            if let Some(t) = parse_12h_time(rest.trim(), true) {
                return t;
            }
        }
        if let Some(rest) = s.strip_suffix("am") {
            if let Some(t) = parse_12h_time(rest.trim(), false) {
                return t;
            }
        }
        NaiveTime::parse_from_str(&s, "%H:%M")
            .unwrap_or_else(|_| panic!("Failed to parse seed time: {s}"))
    }

    fn parse_seed_date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .unwrap_or_else(|_| panic!("Failed to parse seed date: {s}"))
    }

    fn seed_reservations() -> Vec<Reservation> {
        let entries: &[(&str, &str, &str, &str, u32)] = &[
            ("REST-ABC123", "Maya Chen", "2026-08-23", "7:00 pm", 2),
            ("REST-ZX90K2", "Jean Martin", "2026-06-12", "8:00 pm", 4),
            ("REST-2026A1", "Priya Singh", "2026-07-08", "7:30 pm", 3),
            ("REST-7F4K2A", "Noah Davis", "2026-05-20", "6:45 pm", 6),
            ("REST-MN45QP", "Alice Brown", "2026-09-15", "12:00 pm", 2),
            ("REST-9X8Y7Z", "Sam Wilson", "2026-08-01", "9:00 pm", 5),
            ("REST-BOOK42", "Omar Khan", "2026-06-30", "7:15 pm", 8),
            ("REST-CXL777", "Lena Smith", "2026-07-25", "6:00 pm", 1),
            ("REST-A1B2C3", "Alex Carter", "2026-10-03", "8:30 pm", 10),
            ("REST-TABLE9", "Nina Patel", "2026-11-12", "1:00 pm", 4),
        ];
        entries
            .iter()
            .map(|(reference, name, date, time, people)| {
                Reservation::new(
                    reference,
                    name,
                    Self::parse_seed_date(date),
                    Self::parse_seed_time(time),
                    *people,
                )
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Availability & table allocation
    // -----------------------------------------------------------------------

    /// Returns `true` when the requested start time is within opening hours
    /// (the session must start at or after `opening` and end by `closing`).
    fn is_open(&self, time: NaiveTime) -> bool {
        if time < self.config.opening {
            return false;
        }
        // Compute slot end; if it overflows midnight the slot is outside opening hours
        let slot_end_secs = time.num_seconds_from_midnight() as i64
            + (self.config.slot_mins as i64 * 60);
        let closing_secs = self.config.closing.num_seconds_from_midnight() as i64;
        slot_end_secs <= closing_secs
    }

    /// Count how many of each table capacity tier are consumed by existing overlapping reservations.
    fn booked_tables(
        &self,
        reservations: &[Reservation],
        date: NaiveDate,
        time: NaiveTime,
    ) -> std::collections::HashMap<u32, u32> {
        let slot_secs = (self.config.slot_mins * 60) as i64;
        let req_start = NaiveDateTime::new(date, time);
        let req_end = req_start + Duration::seconds(slot_secs);

        // Build mutable available counts to simulate allocation
        let mut used: std::collections::HashMap<u32, u32> = self
            .config
            .tables
            .iter()
            .map(|t| (t.capacity, 0u32))
            .collect();

        // Sort tables ascending for smallest-sufficient allocation
        let mut tables_asc: Vec<(u32, u32)> = self
            .config
            .tables
            .iter()
            .map(|t| (t.capacity, t.count))
            .collect();
        tables_asc.sort_by_key(|(cap, _)| *cap);

        for res in reservations.iter().filter(|r| r.date == date) {
            let res_start = NaiveDateTime::new(res.date, res.time);
            let res_end = res_start + Duration::seconds(slot_secs);
            if req_start < res_end && res_start < req_end {
                // Allocate tables for this reservation using greedy largest-first
                let mut remaining = res.people_count;
                let mut tables_desc = tables_asc.clone();
                tables_desc.sort_by(|a, b| b.0.cmp(&a.0));
                for (cap, _) in &tables_desc {
                    if remaining == 0 {
                        break;
                    }
                    let available = tables_asc
                        .iter()
                        .find(|(c, _)| c == cap)
                        .map(|(_, cnt)| cnt.saturating_sub(*used.get(cap).unwrap_or(&0)))
                        .unwrap_or(0);
                    if available == 0 {
                        continue;
                    }
                    let needed = ((remaining + cap - 1) / cap).min(available);
                    *used.entry(*cap).or_insert(0) += needed;
                    remaining = remaining.saturating_sub(needed * cap);
                }
            }
        }

        used
    }

    /// Try to allocate tables for `people` using available capacity.
    fn can_seat(
        &self,
        reservations: &[Reservation],
        date: NaiveDate,
        time: NaiveTime,
        people: u32,
    ) -> bool {
        let booked = self.booked_tables(reservations, date, time);

        // Build remaining availability sorted descending for greedy allocation
        let mut available: Vec<(u32, u32)> = self
            .config
            .tables
            .iter()
            .map(|t| {
                let used = booked.get(&t.capacity).copied().unwrap_or(0);
                (t.capacity, t.count.saturating_sub(used))
            })
            .filter(|(_, avail)| *avail > 0)
            .collect();

        available.sort_by(|a, b| b.0.cmp(&a.0));

        let mut remaining = people;
        for (cap, count) in &available {
            if remaining == 0 {
                break;
            }
            let needed = (remaining + cap - 1) / cap;
            let used = needed.min(*count);
            remaining = remaining.saturating_sub(used * cap);
        }

        remaining == 0
    }

    /// Find the next slot (within the next 7 days) where `people` can be seated.
    fn next_available_slot(
        &self,
        reservations: &[Reservation],
        from_date: NaiveDate,
        from_time: NaiveTime,
        people: u32,
    ) -> Option<NaiveDateTime> {
        let slot_step = Duration::minutes(self.config.slot_mins as i64);
        let mut candidate = NaiveDateTime::new(from_date, from_time) + slot_step;

        for _ in 0..(7 * 24 * 60 / self.config.slot_mins) {
            let d = candidate.date();
            let t = candidate.time();
            if self.is_open(t) && self.can_seat(reservations, d, t, people) {
                return Some(candidate);
            }
            candidate = candidate + slot_step;
        }
        None
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn reservations(&self) -> std::sync::MutexGuard<'_, Vec<Reservation>> {
        self.reservations.lock().expect("reservations mutex poisoned")
    }

    fn generate_reference(reservations: &[Reservation]) -> String {
        let next_index = reservations.len() + 1;
        format!("REST-{next_index:06X}")
    }

    fn parse_price_amount(amount: &str) -> Option<u32> {
        let cleaned = amount
            .replace("euros", "")
            .replace("dollars", "")
            .replace('$', "")
            .trim()
            .to_string();
        cleaned.parse::<u32>().ok()
    }

    fn is_below(comparator: &str) -> bool {
        matches!(
            comparator.to_lowercase().as_str(),
            "under" | "less than" | "below"
        ) || comparator.to_lowercase().starts_with("di bawah")
            || comparator.to_lowercase().starts_with("kurang")
    }

    fn is_above(comparator: &str) -> bool {
        matches!(
            comparator.to_lowercase().as_str(),
            "greater than" | "more than" | "over"
        ) || comparator.to_lowercase().starts_with("lebih")
            || comparator.to_lowercase().starts_with("di atas")
    }

    fn filter_by_price<'a>(items: &'a [MenuItem], filter: &PriceFilter) -> Vec<&'a MenuItem> {
        let Some(threshold) = Self::parse_price_amount(&filter.amount) else {
            return vec![];
        };
        if Self::is_below(&filter.comparator) {
            items.iter().filter(|i| i.price_euros < threshold).collect()
        } else if Self::is_above(&filter.comparator) {
            items.iter().filter(|i| i.price_euros > threshold).collect()
        } else {
            vec![]
        }
    }

    fn facility_matches(candidate: &str, requested: &str) -> bool {
        fn normalize(v: &str) -> String {
            v.to_lowercase()
                .replace("seats", "seating")
                .replace("seat", "seating")
                .replace('-', " ")
        }
        let c = normalize(candidate);
        let r = normalize(requested);
        c.contains(&r) || r.contains(&c)
    }

    fn find_menu_internal(&self, query: MenuQuery) -> String {
        if let Some(filter) = query.price_filter {
            let matches = Self::filter_by_price(&self.menu, &filter);
            if matches.is_empty() {
                return "no_results:".to_string();
            }
            let names: Vec<String> = matches
                .iter()
                .map(|item| format!("{} (EUR {})", item.name, item.price_euros))
                .collect();
            return format!("price_results:{}", names.join(", "));
        }
        if let Some(item_name) = query.price_item {
            if let Some(item) = self
                .menu
                .iter()
                .find(|i| i.name.to_lowercase().contains(&item_name.to_lowercase()))
            {
                return format!(
                    "item_found:{}|{}|{}",
                    item.name,
                    item.price_euros,
                    item.allergens.join(",")
                );
            }
            return "item_not_found:".to_string();
        }
        let names: Vec<String> = self
            .menu
            .iter()
            .map(|item| format!("{} (EUR {})", item.name, item.price_euros))
            .collect();
        format!("full_menu:{}", names.join(", "))
    }

    fn find_menu_dietary_internal(&self, query: MenuDietaryQuery) -> String {
        if let Some(requirement) = query.dietary_requirement {
            let matches: Vec<&MenuItem> =
                self.menu.iter().filter(|i| i.has_dietary(&requirement)).collect();
            if matches.is_empty() {
                return format!("no_dietary:{requirement}");
            }
            let names: Vec<String> = matches.iter().map(|i| i.name.clone()).collect();
            return format!("dietary_results:{}|{}", requirement, names.join(", "));
        }
        "dietary_no_filter:vegan, vegetarian, halal, gluten-free, dairy-free, nut-free, low-salt"
            .to_string()
    }

    fn find_menu_item_details_internal(&self, query: MenuItemDetailsQuery) -> String {
        match (query.menu_item, query.allergen) {
            (Some(item_name), Some(allergen)) => {
                if let Some(item) = self
                    .menu
                    .iter()
                    .find(|i| i.name.to_lowercase().contains(&item_name.to_lowercase()))
                {
                    if item.has_allergen(&allergen) {
                        return format!("contains:{}|{}", item_name, allergen);
                    }
                    return format!("not_contains:{}|{}", item_name, allergen);
                }
                format!("item_unknown:{item_name}")
            }
            (Some(item_name), None) => {
                if let Some(item) = self
                    .menu
                    .iter()
                    .find(|i| i.name.to_lowercase().contains(&item_name.to_lowercase()))
                {
                    let dietary = if item.dietary.is_empty() { "none".to_string() } else { item.dietary.join(", ") };
                    let allergens = if item.allergens.is_empty() { "none".to_string() } else { item.allergens.join(", ") };
                    return format!("item_details:{}|{}|{}|{}", item.name, item.price_euros, dietary, allergens);
                }
                format!("item_unknown:{item_name}")
            }
            (None, Some(allergen)) => {
                let matches: Vec<&MenuItem> =
                    self.menu.iter().filter(|i| i.has_allergen(&allergen)).collect();
                if matches.is_empty() {
                    return format!("no_allergen_match:{allergen}");
                }
                let names: Vec<String> = matches.iter().map(|i| i.name.clone()).collect();
                format!("allergen_found:{}|{}", allergen, names.join(", "))
            }
            (None, None) => "details_no_filter:".to_string(),
        }
    }

    fn find_location_internal(&self, query: LocationQuery) -> String {
        let address = "12 Rue de la Paix, 75001 Paris - near the city center, by the river";
        if let Some(location) = query.near {
            let normalized = location.to_lowercase();
            let is_near = matches!(
                normalized.as_str(),
                "city center" | "downtown" | "by the river" | "near the station" | "main branch"
            );
            if is_near {
                return format!("near_confirmed:{}|{}", location, address);
            }
            return format!("near_denied:{}|{}", location, address);
        }
        format!("address:{address}")
    }

    fn find_payment_methods_internal(&self, query: PaymentMethodQuery) -> String {
        let all_methods = self.payment_methods.join(", ");
        if let Some(method) = query.method {
            let accepted = self
                .payment_methods
                .iter()
                .any(|m| m.to_lowercase().contains(&method.to_lowercase()));
            if accepted {
                return format!("method_accepted:{}|{}", method, all_methods);
            }
            return format!("method_not_accepted:{}|{}", method, all_methods);
        }
        format!("all_methods:{all_methods}")
    }

    fn find_price_internal(&self, query: PriceQuery) -> String {
        if let Some(filter) = query.price_filter {
            let matches = Self::filter_by_price(&self.menu, &filter);
            if matches.is_empty() {
                return format!("no_price_results:{}|{}", filter.comparator, filter.amount);
            }
            let names: Vec<String> = matches
                .iter()
                .map(|item| format!("{} (EUR {})", item.name, item.price_euros))
                .collect();
            return format!("price_results:{}|{}|{}", filter.comparator, filter.amount, names.join(", "));
        }
        if let Some(item_name) = query.item {
            if let Some(item) = self
                .menu
                .iter()
                .find(|i| i.name.to_lowercase().contains(&item_name.to_lowercase()))
            {
                return format!("item_price:{}|{}", item.name, item.price_euros);
            }
            return format!("item_not_found:{item_name}");
        }
        "price_general:our prices range from EUR 6 to EUR 75".to_string()
    }

    fn find_event_info_internal(&self, query: EventQuery) -> String {
        let event_spaces = ["terrace", "private room"];
        if let Some(location) = query.location {
            let available = event_spaces.iter().any(|s| location.to_lowercase().contains(s));
            if available {
                return format!(
                    "event_space_available:{}|Contact us at events@example.com to book.",
                    location
                );
            }
            return format!(
                "event_space_unavailable:{}|We have a terrace and a private room available for events.",
                location
            );
        }
        "event_info:We host birthday parties, corporate events, and private dinners. Spaces: terrace and private room. Contact events@example.com.".to_string()
    }

    fn find_facility_info_internal(&self, query: FacilityQuery) -> String {
        if let Some(facility) = query.facility {
            let available = self
                .facilities
                .iter()
                .any(|f| Self::facility_matches(f, &facility));
            if available {
                return format!("facility_available:{facility}");
            }
            return format!("facility_unavailable:{facility}");
        }
        format!("all_facilities:{}", self.facilities.join(", "))
    }

    fn check_reservation_internal(&self, query: ReservationLookupQuery) -> String {
        if let Some(reference) = query.reference {
            if let Some(res) = self
                .reservations()
                .iter()
                .find(|r| r.reference.eq_ignore_ascii_case(&reference))
            {
                return format!(
                    "found:{}|{}|{}|{}|{}",
                    res.reference,
                    res.name,
                    res.date,
                    res.time.format("%H:%M"),
                    res.people_count
                );
            }
            return format!("not_found:{reference}");
        }
        if let Some(name) = query.name {
            let normalized = name.to_lowercase();
            let matches = self
                .reservations()
                .iter()
                .filter(|r| r.name.to_lowercase() == normalized)
                .map(|r| {
                    format!(
                        "{}~{}~{}~{}",
                        r.reference,
                        r.date,
                        r.time.format("%H:%M"),
                        r.people_count
                    )
                })
                .collect::<Vec<_>>();
            if matches.is_empty() {
                return format!("name_not_found:{name}");
            }
            return format!("listed:{name}|{}", matches.join(";"));
        }
        "no_reference_or_name:".to_string()
    }
}

impl Default for RestaurantService {
    fn default() -> Self {
        Self::new()
    }
}

impl RestaurantInformationUseCase for RestaurantService {
    fn get_opening_hours(&self) -> String {
        format!(
            "Mon-Sun {} - {}",
            self.config.opening.format("%I:%M %P"),
            self.config.closing.format("%I:%M %P")
        )
    }

    fn find_menu(&self, query: MenuQuery) -> String { self.find_menu_internal(query) }
    fn find_menu_dietary(&self, query: MenuDietaryQuery) -> String { self.find_menu_dietary_internal(query) }
    fn find_menu_item_details(&self, query: MenuItemDetailsQuery) -> String { self.find_menu_item_details_internal(query) }
    fn find_location(&self, query: LocationQuery) -> String { self.find_location_internal(query) }
    fn get_contact(&self) -> String { "contact:+33123456789|booking@example.com".to_string() }
    fn find_payment_methods(&self, query: PaymentMethodQuery) -> String { self.find_payment_methods_internal(query) }
    fn find_price(&self, query: PriceQuery) -> String { self.find_price_internal(query) }
    fn get_takeaway_info(&self) -> String { "takeaway:yes|We offer takeaway and delivery. Order by phone or at the counter.".to_string() }
    fn find_event_info(&self, query: EventQuery) -> String { self.find_event_info_internal(query) }
    fn find_facility_info(&self, query: FacilityQuery) -> String { self.find_facility_info_internal(query) }
    fn get_accessibility_info(&self) -> String { "accessibility:yes|The restaurant is wheelchair accessible with step-free access at the main entrance. Strollers are welcome.".to_string() }
    fn get_entertainment_info(&self) -> String { "entertainment:yes|We have live music every Friday and Saturday evening. A DJ performs on Saturday nights.".to_string() }
}

impl RestaurantReservationUseCase for RestaurantService {
    fn create_reservation(&self, query: ReservationCreateQuery) -> Result<String, ReservationError> {
        if !self.is_open(query.time) {
            return Err(ReservationError::RestaurantClosed);
        }

        let mut reservations = self.reservations();
        if !self.can_seat(&reservations, query.date, query.time, query.people_count) {
            let next_slot = self.next_available_slot(
                &reservations,
                query.date,
                query.time,
                query.people_count,
            );
            return Err(ReservationError::NoAvailability { next_slot });
        }

        let reference = Self::generate_reference(&reservations);
        reservations.push(Reservation::new(
            &reference,
            &query.name,
            query.date,
            query.time,
            query.people_count,
        ));
        Ok(format!("created:{reference}"))
    }

    fn check_reservation(&self, query: ReservationLookupQuery) -> String {
        self.check_reservation_internal(query)
    }
}

// ---------------------------------------------------------------------------
// Shared time parsing helper (used by seed + resolve_time in conversation)
// ---------------------------------------------------------------------------

pub(crate) fn parse_12h_time(rest: &str, pm: bool) -> Option<NaiveTime> {
    let (h_str, m_str) = if let Some((h, m)) = rest.split_once(':') {
        (h, m)
    } else {
        (rest, "0")
    };
    let h: u32 = h_str.trim().parse().ok()?;
    let m: u32 = m_str.trim().parse().ok()?;
    if h > 12 || m > 59 {
        return None;
    }
    let hour = match (h, pm) {
        (12, true) => 12,
        (12, false) => 0,
        (h, true) => h + 12,
        (h, false) => h,
    };
    NaiveTime::from_hms_opt(hour, m, 0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> RestaurantService {
        RestaurantService::new()
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn time(h: u32, min: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, min, 0).unwrap()
    }

    #[test]
    fn get_opening_hours_returns_hours() {
        assert!(service().get_opening_hours().contains("11"));
    }

    #[test]
    fn create_reservation_succeeds_when_slot_is_available() {
        let svc = service();
        let result = svc.create_reservation(ReservationCreateQuery {
            name: "Test User".to_string(),
            date: date(2099, 1, 15),
            time: time(19, 0),
            people_count: 2,
        });
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("created:"));
    }

    #[test]
    fn create_reservation_fails_when_restaurant_is_closed() {
        let svc = service();
        let result = svc.create_reservation(ReservationCreateQuery {
            name: "Test User".to_string(),
            date: date(2099, 1, 15),
            time: time(23, 0), // after closing
            people_count: 2,
        });
        assert_eq!(result, Err(ReservationError::RestaurantClosed));
    }

    #[test]
    fn create_reservation_fails_when_too_many_people() {
        let svc = service();
        // Total capacity is 2*6 + 3*4 + 3*2 = 12+12+6 = 30; requesting > 30 should fail
        let result = svc.create_reservation(ReservationCreateQuery {
            name: "Big Party".to_string(),
            date: date(2099, 1, 15),
            time: time(19, 0),
            people_count: 35,
        });
        assert!(matches!(result, Err(ReservationError::NoAvailability { .. })));
    }

    #[test]
    fn next_available_slot_is_suggested_when_slot_full() {
        let svc = service();
        // Fill all tables at a specific slot by reserving the full capacity in one go
        let d = date(2099, 6, 1);
        let t = time(19, 0);

        // Create a reservation that fills all tables
        let _ = svc.create_reservation(ReservationCreateQuery {
            name: "Group A".to_string(),
            date: d,
            time: t,
            people_count: 30, // fills everything
        });

        let result = svc.create_reservation(ReservationCreateQuery {
            name: "Group B".to_string(),
            date: d,
            time: t,
            people_count: 2,
        });

        match result {
            Err(ReservationError::NoAvailability { next_slot: Some(_) }) => {}
            other => panic!("Expected NoAvailability with next_slot, got {:?}", other),
        }
    }

    #[test]
    fn check_reservation_known_reference_returns_found() {
        let result = service().check_reservation(ReservationLookupQuery {
            reference: Some("REST-ABC123".to_string()),
            name: None,
        });
        assert!(result.starts_with("found:"));
        assert!(result.contains("Maya Chen"));
    }

    #[test]
    fn check_reservation_unknown_reference_returns_not_found() {
        assert!(service()
            .check_reservation(ReservationLookupQuery { reference: Some("REST-UNKNOWN".to_string()), name: None })
            .starts_with("not_found:"));
    }

    #[test]
    fn check_reservation_no_reference_returns_no_reference() {
        assert!(service()
            .check_reservation(ReservationLookupQuery { reference: None, name: None })
            .starts_with("no_reference_or_name:"));
    }

    #[test]
    fn check_reservation_by_name_returns_list() {
        let result = service().check_reservation(ReservationLookupQuery {
            reference: None,
            name: Some("Maya Chen".to_string()),
        });
        assert!(result.starts_with("listed:Maya Chen|"));
        assert!(result.contains("REST-ABC123"));
    }

    #[test]
    fn get_menu_dietary_vegan_returns_matching_items() {
        let result = service().find_menu_dietary(MenuDietaryQuery {
            dietary_requirement: Some("vegan".to_string()),
        });
        assert!(result.starts_with("dietary_results:"));
        assert!(result.contains("salad"));
        assert!(result.contains("vegan curry"));
    }

    #[test]
    fn get_menu_item_details_contains_allergen() {
        assert_eq!(
            service().find_menu_item_details(MenuItemDetailsQuery {
                menu_item: Some("pizza".to_string()),
                allergen: Some("gluten".to_string()),
            }),
            "contains:pizza|gluten"
        );
    }

    #[test]
    fn get_payment_methods_accepted() {
        assert!(service()
            .find_payment_methods(PaymentMethodQuery { method: Some("credit card".to_string()) })
            .starts_with("method_accepted:"));
    }

    #[test]
    fn get_facility_available() {
        assert_eq!(
            service().find_facility_info(FacilityQuery { facility: Some("wifi".to_string()) }),
            "facility_available:wifi"
        );
    }
}





