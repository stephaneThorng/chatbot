use crate::core::restaurant::application::port::inbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    EventQuery, FacilityQuery, LocationQuery, MenuDietaryQuery, MenuItemDetailsQuery, MenuQuery,
    PaymentMethodQuery, PriceFilter, PriceQuery, ReservationCreateQuery, ReservationLookupQuery,
};
use crate::core::restaurant::application::port::inbound::restaurant_reservation_port::RestaurantReservationPort;
use crate::core::restaurant::domain::model::{MenuItem, Reservation};
use std::sync::Mutex;

/// In-memory implementation of restaurant read capabilities for v1.
/// All data is static and constructed at `new()` time.
pub struct RestaurantAdapter {
    menu: Vec<MenuItem>,
    reservations: Mutex<Vec<Reservation>>,
    payment_methods: Vec<String>,
    facilities: Vec<String>,
}

impl RestaurantAdapter {
    pub fn new() -> Self {
        Self {
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

    fn seed_menu() -> Vec<MenuItem> {
        vec![
            MenuItem::new("pizza", &["vegetarian"], &["gluten", "dairy"], 12),
            MenuItem::new(
                "salad",
                &[
                    "vegan",
                    "vegetarian",
                    "gluten-free",
                    "dairy-free",
                    "nut-free",
                ],
                &[],
                8,
            ),
            MenuItem::new(
                "chocolate cake",
                &["vegetarian"],
                &["gluten", "dairy", "eggs"],
                6,
            ),
            MenuItem::new("fried rice", &["gluten-free"], &["eggs", "soy"], 10),
            MenuItem::new(
                "vegetarian pasta",
                &["vegetarian"],
                &["gluten", "dairy", "eggs"],
                11,
            ),
            MenuItem::new(
                "seafood soup",
                &["gluten-free", "dairy-free"],
                &["shellfish", "soy"],
                14,
            ),
            MenuItem::new(
                "beef burger",
                &[],
                &["gluten", "dairy", "eggs", "sesame"],
                14,
            ),
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
            MenuItem::new(
                "kids pasta",
                &["vegetarian"],
                &["gluten", "dairy", "eggs"],
                8,
            ),
            MenuItem::new("set menu", &[], &["gluten", "dairy"], 35),
            MenuItem::new("lunch special", &[], &["gluten"], 15),
            MenuItem::new("kids menu", &["vegetarian"], &["gluten", "dairy"], 10),
            MenuItem::new(
                "breakfast menu",
                &["vegetarian"],
                &["gluten", "dairy", "eggs"],
                12,
            ),
            MenuItem::new("family menu", &[], &["gluten", "dairy"], 60),
            MenuItem::new("tasting menu", &[], &["gluten", "dairy", "shellfish"], 75),
            MenuItem::new(
                "dessert menu",
                &["vegetarian"],
                &["gluten", "dairy", "eggs"],
                18,
            ),
        ]
    }

    fn seed_reservations() -> Vec<Reservation> {
        vec![
            Reservation::new("REST-ABC123", "Maya Chen", "2026-08-23", "7:00 pm", 2),
            Reservation::new("REST-ZX90K2", "Jean Martin", "2026-06-12", "8:00 pm", 4),
            Reservation::new("REST-2026A1", "Priya Singh", "2026-07-08", "7:30 pm", 3),
            Reservation::new("REST-7F4K2A", "Noah Davis", "2026-05-20", "6:45 pm", 6),
            Reservation::new("REST-MN45QP", "Alice Brown", "2026-09-15", "12:00 pm", 2),
            Reservation::new("REST-9X8Y7Z", "Sam Wilson", "2026-08-01", "9:00 pm", 5),
            Reservation::new("REST-BOOK42", "Omar Khan", "2026-06-30", "7:15 pm", 8),
            Reservation::new("REST-CXL777", "Lena Smith", "2026-07-25", "6:00 pm", 1),
            Reservation::new("REST-A1B2C3", "Alex Carter", "2026-10-03", "8:30 pm", 10),
            Reservation::new("REST-TABLE9", "Nina Patel", "2026-11-12", "1:00 pm", 4),
        ]
    }

    fn reservations(&self) -> std::sync::MutexGuard<'_, Vec<Reservation>> {
        self.reservations
            .lock()
            .expect("restaurant reservations mutex poisoned")
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
            items
                .iter()
                .filter(|menu_item| menu_item.price_euros < threshold)
                .collect()
        } else if Self::is_above(&filter.comparator) {
            items
                .iter()
                .filter(|menu_item| menu_item.price_euros > threshold)
                .collect()
        } else {
            vec![]
        }
    }

    fn facility_matches(candidate: &str, requested: &str) -> bool {
        fn normalize(value: &str) -> String {
            value
                .to_lowercase()
                .replace("seats", "seating")
                .replace("seat", "seating")
                .replace('-', " ")
        }

        let candidate = normalize(candidate);
        let requested = normalize(requested);
        candidate.contains(&requested) || requested.contains(&candidate)
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
            if let Some(item) = self.menu.iter().find(|menu_item| {
                menu_item
                    .name
                    .to_lowercase()
                    .contains(&item_name.to_lowercase())
            }) {
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
            let matches: Vec<&MenuItem> = self
                .menu
                .iter()
                .filter(|menu_item| menu_item.has_dietary(&requirement))
                .collect();
            if matches.is_empty() {
                return format!("no_dietary:{requirement}");
            }
            let names: Vec<String> = matches.iter().map(|item| item.name.clone()).collect();
            return format!("dietary_results:{}|{}", requirement, names.join(", "));
        }

        "dietary_no_filter:vegan, vegetarian, halal, gluten-free, dairy-free, nut-free, low-salt"
            .to_string()
    }

    fn find_menu_item_details_internal(&self, query: MenuItemDetailsQuery) -> String {
        match (query.menu_item, query.allergen) {
            (Some(item_name), Some(allergen)) => {
                if let Some(item) = self.menu.iter().find(|menu_item| {
                    menu_item
                        .name
                        .to_lowercase()
                        .contains(&item_name.to_lowercase())
                }) {
                    if item.has_allergen(&allergen) {
                        return format!("contains:{}|{}", item_name, allergen);
                    }
                    return format!("not_contains:{}|{}", item_name, allergen);
                }
                format!("item_unknown:{item_name}")
            }
            (Some(item_name), None) => {
                if let Some(item) = self.menu.iter().find(|menu_item| {
                    menu_item
                        .name
                        .to_lowercase()
                        .contains(&item_name.to_lowercase())
                }) {
                    let dietary = if item.dietary.is_empty() {
                        "none".to_string()
                    } else {
                        item.dietary.join(", ")
                    };
                    let allergens = if item.allergens.is_empty() {
                        "none".to_string()
                    } else {
                        item.allergens.join(", ")
                    };
                    return format!(
                        "item_details:{}|{}|{}|{}",
                        item.name, item.price_euros, dietary, allergens
                    );
                }
                format!("item_unknown:{item_name}")
            }
            (None, Some(allergen)) => {
                let matches: Vec<&MenuItem> = self
                    .menu
                    .iter()
                    .filter(|menu_item| menu_item.has_allergen(&allergen))
                    .collect();
                if matches.is_empty() {
                    return format!("no_allergen_match:{allergen}");
                }
                let names: Vec<String> = matches.iter().map(|item| item.name.clone()).collect();
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
                .any(|candidate| candidate.to_lowercase().contains(&method.to_lowercase()));
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
            return format!(
                "price_results:{}|{}|{}",
                filter.comparator,
                filter.amount,
                names.join(", ")
            );
        }

        if let Some(item_name) = query.item {
            if let Some(item) = self.menu.iter().find(|menu_item| {
                menu_item
                    .name
                    .to_lowercase()
                    .contains(&item_name.to_lowercase())
            }) {
                return format!("item_price:{}|{}", item.name, item.price_euros);
            }
            return format!("item_not_found:{item_name}");
        }

        "price_general:our prices range from EUR 6 to EUR 75".to_string()
    }

    fn find_event_info_internal(&self, query: EventQuery) -> String {
        let event_spaces = ["terrace", "private room"];
        if let Some(location) = query.location {
            let available = event_spaces
                .iter()
                .any(|space| location.to_lowercase().contains(space));
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
                .any(|candidate| Self::facility_matches(candidate, &facility));
            if available {
                return format!("facility_available:{facility}");
            }
            return format!("facility_unavailable:{facility}");
        }

        format!("all_facilities:{}", self.facilities.join(", "))
    }

    fn check_reservation_internal(&self, query: ReservationLookupQuery) -> String {
        if let Some(reference) = query.reference {
            if let Some(reservation) = self
                .reservations()
                .iter()
                .find(|candidate| candidate.reference.eq_ignore_ascii_case(&reference))
            {
                return format!(
                    "found:{}|{}|{}|{}|{}",
                    reservation.reference,
                    reservation.name,
                    reservation.date,
                    reservation.time,
                    reservation.people_count
                );
            }
            return format!("not_found:{reference}");
        }

        if let Some(name) = query.name {
            let normalized = name.to_lowercase();
            let matches = self
                .reservations()
                .iter()
                .filter(|candidate| candidate.name.to_lowercase() == normalized)
                .map(|reservation| {
                    format!(
                        "{}~{}~{}~{}",
                        reservation.reference,
                        reservation.date,
                        reservation.time,
                        reservation.people_count
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

impl Default for RestaurantAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RestaurantInformationPort for RestaurantAdapter {
    fn get_opening_hours(&self) -> String {
        "Mon-Sun 11:00 am - 10:00 pm".to_string()
    }

    fn find_menu(&self, query: MenuQuery) -> String {
        self.find_menu_internal(query)
    }

    fn find_menu_dietary(&self, query: MenuDietaryQuery) -> String {
        self.find_menu_dietary_internal(query)
    }

    fn find_menu_item_details(&self, query: MenuItemDetailsQuery) -> String {
        self.find_menu_item_details_internal(query)
    }

    fn find_location(&self, query: LocationQuery) -> String {
        self.find_location_internal(query)
    }

    fn get_contact(&self) -> String {
        "contact:+33123456789|booking@example.com".to_string()
    }

    fn find_payment_methods(&self, query: PaymentMethodQuery) -> String {
        self.find_payment_methods_internal(query)
    }

    fn find_price(&self, query: PriceQuery) -> String {
        self.find_price_internal(query)
    }

    fn get_takeaway_info(&self) -> String {
        "takeaway:yes|We offer takeaway and delivery. Order by phone or at the counter.".to_string()
    }

    fn find_event_info(&self, query: EventQuery) -> String {
        self.find_event_info_internal(query)
    }

    fn find_facility_info(&self, query: FacilityQuery) -> String {
        self.find_facility_info_internal(query)
    }

    fn get_accessibility_info(&self) -> String {
        "accessibility:yes|The restaurant is wheelchair accessible with step-free access at the main entrance. Strollers are welcome.".to_string()
    }

    fn get_entertainment_info(&self) -> String {
        "entertainment:yes|We have live music every Friday and Saturday evening. A DJ performs on Saturday nights.".to_string()
    }
}

impl RestaurantReservationPort for RestaurantAdapter {
    fn create_reservation(&self, query: ReservationCreateQuery) -> String {
        let mut reservations = self.reservations();
        let reference = Self::generate_reference(&reservations);
        reservations.push(Reservation::new(
            &reference,
            &query.name,
            &query.date,
            &query.time,
            query.people_count,
        ));
        format!("created:{reference}")
    }

    fn check_reservation(&self, query: ReservationLookupQuery) -> String {
        self.check_reservation_internal(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adapter() -> RestaurantAdapter {
        RestaurantAdapter::new()
    }

    #[test]
    fn get_opening_hours_returns_hours() {
        assert!(adapter().get_opening_hours().contains("11:00"));
    }

    #[test]
    fn check_reservation_known_reference_returns_found() {
        let result = adapter().check_reservation(ReservationLookupQuery {
            reference: Some("REST-ABC123".to_string()),
            name: None,
        });
        assert!(result.starts_with("found:"));
        assert!(result.contains("Maya Chen"));
    }

    #[test]
    fn check_reservation_unknown_reference_returns_not_found() {
        let result = adapter().check_reservation(ReservationLookupQuery {
            reference: Some("REST-UNKNOWN".to_string()),
            name: None,
        });
        assert!(result.starts_with("not_found:"));
    }

    #[test]
    fn check_reservation_no_reference_returns_no_reference() {
        assert!(
            adapter()
                .check_reservation(ReservationLookupQuery {
                    reference: None,
                    name: None
                })
                .starts_with("no_reference_or_name:")
        );
    }

    #[test]
    fn check_reservation_by_name_returns_list() {
        let result = adapter().check_reservation(ReservationLookupQuery {
            reference: None,
            name: Some("Maya Chen".to_string()),
        });
        assert!(result.starts_with("listed:Maya Chen|"));
        assert!(result.contains("REST-ABC123"));
    }

    #[test]
    fn get_menu_dietary_vegan_returns_matching_items() {
        let result = adapter().find_menu_dietary(MenuDietaryQuery {
            dietary_requirement: Some("vegan".to_string()),
        });
        assert!(result.starts_with("dietary_results:"));
        assert!(result.contains("salad"));
        assert!(result.contains("vegan curry"));
    }

    #[test]
    fn get_menu_item_details_contains_allergen() {
        let result = adapter().find_menu_item_details(MenuItemDetailsQuery {
            menu_item: Some("pizza".to_string()),
            allergen: Some("gluten".to_string()),
        });
        assert_eq!(result, "contains:pizza|gluten");
    }

    #[test]
    fn get_menu_item_details_not_contains_allergen() {
        let result = adapter().find_menu_item_details(MenuItemDetailsQuery {
            menu_item: Some("salad".to_string()),
            allergen: Some("gluten".to_string()),
        });
        assert_eq!(result, "not_contains:salad|gluten");
    }

    #[test]
    fn get_payment_methods_accepted() {
        let result = adapter().find_payment_methods(PaymentMethodQuery {
            method: Some("credit card".to_string()),
        });
        assert!(result.starts_with("method_accepted:"));
    }

    #[test]
    fn get_payment_methods_not_accepted() {
        let result = adapter().find_payment_methods(PaymentMethodQuery {
            method: Some("crypto".to_string()),
        });
        assert!(result.starts_with("method_not_accepted:"));
    }

    #[test]
    fn get_facility_available() {
        let result = adapter().find_facility_info(FacilityQuery {
            facility: Some("wifi".to_string()),
        });
        assert_eq!(result, "facility_available:wifi");
    }

    #[test]
    fn get_facility_unavailable() {
        let result = adapter().find_facility_info(FacilityQuery {
            facility: Some("swimming pool".to_string()),
        });
        assert!(result.starts_with("facility_unavailable:"));
    }
}
