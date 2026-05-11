use crate::core::restaurant::application::port::inbound::restaurant_trait::RestaurantPort;
use crate::core::restaurant::domain::model::{MenuItem, Reservation};

/// In-memory implementation of [`RestaurantPort`] for v1.
/// All data is static and constructed at `new()` time.
pub struct RestaurantAdapter {
    menu: Vec<MenuItem>,
    reservations: Vec<Reservation>,
    payment_methods: Vec<String>,
    facilities: Vec<String>,
}

impl RestaurantAdapter {
    pub fn new() -> Self {
        Self {
            menu: Self::seed_menu(),
            reservations: Self::seed_reservations(),
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
            MenuItem::new("salad", &["vegan", "vegetarian", "gluten-free", "dairy-free", "nut-free"], &[], 8),
            MenuItem::new("chocolate cake", &["vegetarian"], &["gluten", "dairy", "eggs"], 6),
            MenuItem::new("fried rice", &["gluten-free"], &["eggs", "soy"], 10),
            MenuItem::new("vegetarian pasta", &["vegetarian"], &["gluten", "dairy", "eggs"], 11),
            MenuItem::new("seafood soup", &["gluten-free", "dairy-free"], &["shellfish", "soy"], 14),
            MenuItem::new("beef burger", &[], &["gluten", "dairy", "eggs", "sesame"], 14),
            MenuItem::new("chicken satay", &["halal", "gluten-free", "dairy-free"], &["peanuts", "soy"], 13),
            MenuItem::new("vegan curry", &["vegan", "vegetarian", "halal", "gluten-free", "dairy-free"], &["soy"], 11),
            MenuItem::new("kids pasta", &["vegetarian"], &["gluten", "dairy", "eggs"], 8),
            // price items (menus)
            MenuItem::new("set menu", &[], &["gluten", "dairy"], 35),
            MenuItem::new("lunch special", &[], &["gluten"], 15),
            MenuItem::new("kids menu", &["vegetarian"], &["gluten", "dairy"], 10),
            MenuItem::new("breakfast menu", &["vegetarian"], &["gluten", "dairy", "eggs"], 12),
            MenuItem::new("family menu", &[], &["gluten", "dairy"], 60),
            MenuItem::new("tasting menu", &[], &["gluten", "dairy", "shellfish"], 75),
            MenuItem::new("dessert menu", &["vegetarian"], &["gluten", "dairy", "eggs"], 18),
        ]
    }

    fn seed_reservations() -> Vec<Reservation> {
        vec![
            Reservation::new("REST-ABC123", "Maya Chen", "2026-08-23", 2),
            Reservation::new("REST-ZX90K2", "Jean Martin", "2026-06-12", 4),
            Reservation::new("REST-2026A1", "Priya Singh", "2026-07-08", 3),
            Reservation::new("REST-7F4K2A", "Noah Davis", "2026-05-20", 6),
            Reservation::new("REST-MN45QP", "Alice Brown", "2026-09-15", 2),
            Reservation::new("REST-9X8Y7Z", "Sam Wilson", "2026-08-01", 5),
            Reservation::new("REST-BOOK42", "Omar Khan", "2026-06-30", 8),
            Reservation::new("REST-CXL777", "Lena Smith", "2026-07-25", 1),
            Reservation::new("REST-A1B2C3", "Alex Carter", "2026-10-03", 10),
            Reservation::new("REST-TABLE9", "Nina Patel", "2026-11-12", 4),
        ]
    }

    /// Parse a simple price amount string and extract a numeric value in euros.
    /// Handles: "20 euros", "$30", "15 euros", "25 dollars", "10", "50 euros", "$45".
    fn parse_price_amount(amount: &str) -> Option<u32> {
        let cleaned = amount
            .replace("euros", "")
            .replace("dollars", "")
            .replace('$', "")
            .trim()
            .to_string();
        cleaned.parse::<u32>().ok()
    }

    fn is_below(price: u32, comparator: &str) -> bool {
        matches!(comparator.to_lowercase().as_str(), "under" | "less than" | "below")
            || comparator.to_lowercase().starts_with("di bawah")
            || comparator.to_lowercase().starts_with("kurang")
    }

    fn is_above(price: u32, comparator: &str) -> bool {
        matches!(comparator.to_lowercase().as_str(), "greater than" | "more than" | "over")
            || comparator.to_lowercase().starts_with("lebih")
            || comparator.to_lowercase().starts_with("di atas")
    }

    fn filter_by_price<'a>(
        items: &'a [MenuItem],
        price_comparator: &str,
        threshold: u32,
    ) -> Vec<&'a MenuItem> {
        if Self::is_below(0, price_comparator) {
            items.iter().filter(|m| m.price_euros < threshold).collect()
        } else if Self::is_above(0, price_comparator) {
            items.iter().filter(|m| m.price_euros > threshold).collect()
        } else {
            vec![]
        }
    }
}

impl Default for RestaurantAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RestaurantPort for RestaurantAdapter {
    fn get_opening_hours(&self) -> String {
        "Mon-Sun 11:00 am – 10:00 pm".to_string()
    }

    fn get_menu(&self, price_item: Option<&str>, price_comparator: Option<&str>, price_amount: Option<&str>) -> String {
        if let (Some(comparator), Some(amount_raw)) = (price_comparator, price_amount) {
            if let Some(threshold) = Self::parse_price_amount(amount_raw) {
                let matches: Vec<&MenuItem> = Self::filter_by_price(&self.menu, comparator, threshold);
                if matches.is_empty() {
                    return "no_results:".to_string();
                }
                let names: Vec<String> = matches.iter().map(|m| format!("{} (€{})", m.name, m.price_euros)).collect();
                return format!("price_results:{}", names.join(", "));
            }
        }
        if let Some(item) = price_item {
            let found = self.menu.iter().find(|m| m.name.to_lowercase().contains(&item.to_lowercase()));
            if let Some(m) = found {
                return format!("item_found:{}|{}|{}", m.name, m.price_euros, m.allergens.join(","));
            }
            return "item_not_found:".to_string();
        }
        let names: Vec<String> = self.menu.iter().map(|m| format!("{} (€{})", m.name, m.price_euros)).collect();
        format!("full_menu:{}", names.join(", "))
    }

    fn get_menu_dietary(&self, dietary: Option<&str>) -> String {
        if let Some(req) = dietary {
            let matches: Vec<&MenuItem> = self.menu.iter().filter(|m| m.has_dietary(req)).collect();
            if matches.is_empty() {
                return format!("no_dietary:{}", req);
            }
            let names: Vec<String> = matches.iter().map(|m| m.name.clone()).collect();
            return format!("dietary_results:{}|{}", req, names.join(", "));
        }
        "dietary_no_filter:vegan, vegetarian, halal, gluten-free, dairy-free, nut-free, low-salt".to_string()
    }

    fn get_menu_item_details(&self, menu_item: Option<&str>, allergen: Option<&str>) -> String {
        match (menu_item, allergen) {
            (Some(item), Some(al)) => {
                let found = self.menu.iter().find(|m| m.name.to_lowercase().contains(&item.to_lowercase()));
                if let Some(m) = found {
                    if m.has_allergen(al) {
                        return format!("contains:{}|{}", item, al);
                    } else {
                        return format!("not_contains:{}|{}", item, al);
                    }
                }
                format!("item_unknown:{}", item)
            }
            (Some(item), None) => {
                let found = self.menu.iter().find(|m| m.name.to_lowercase().contains(&item.to_lowercase()));
                if let Some(m) = found {
                    let dietary = if m.dietary.is_empty() { "none".to_string() } else { m.dietary.join(", ") };
                    let allergens = if m.allergens.is_empty() { "none".to_string() } else { m.allergens.join(", ") };
                    return format!("item_details:{}|{}|{}|{}", m.name, m.price_euros, dietary, allergens);
                }
                format!("item_unknown:{}", item)
            }
            (None, Some(al)) => {
                let matches: Vec<&MenuItem> = self.menu.iter().filter(|m| m.has_allergen(al)).collect();
                if matches.is_empty() {
                    return format!("no_allergen_match:{}", al);
                }
                let names: Vec<String> = matches.iter().map(|m| m.name.clone()).collect();
                format!("allergen_found:{}|{}", al, names.join(", "))
            }
            (None, None) => "details_no_filter:".to_string(),
        }
    }

    fn get_location(&self, near: Option<&str>) -> String {
        let address = "12 Rue de la Paix, 75001 Paris – near the city center, by the river";
        if let Some(loc) = near {
            let loc_lower = loc.to_lowercase();
            let is_near = matches!(
                loc_lower.as_str(),
                "city center" | "downtown" | "by the river" | "near the station" | "main branch"
            );
            if is_near {
                return format!("near_confirmed:{}|{}", loc, address);
            }
            return format!("near_denied:{}|{}", loc, address);
        }
        format!("address:{}", address)
    }

    fn get_contact(&self) -> String {
        "contact:+33123456789|booking@example.com".to_string()
    }

    fn get_payment_methods(&self, method: Option<&str>) -> String {
        let all = self.payment_methods.join(", ");
        if let Some(m) = method {
            let accepted = self.payment_methods.iter().any(|p| p.to_lowercase().contains(&m.to_lowercase()));
            if accepted {
                return format!("method_accepted:{}|{}", m, all);
            }
            return format!("method_not_accepted:{}|{}", m, all);
        }
        format!("all_methods:{}", all)
    }

    fn get_price(&self, item: Option<&str>, price_comparator: Option<&str>, price_amount: Option<&str>) -> String {
        if let (Some(comparator), Some(amount_raw)) = (price_comparator, price_amount) {
            if let Some(threshold) = Self::parse_price_amount(amount_raw) {
                let matches = Self::filter_by_price(&self.menu, comparator, threshold);
                if matches.is_empty() {
                    return format!("no_price_results:{}|{}", comparator, amount_raw);
                }
                let names: Vec<String> = matches.iter().map(|m| format!("{} (€{})", m.name, m.price_euros)).collect();
                return format!("price_results:{}|{}|{}", comparator, amount_raw, names.join(", "));
            }
        }
        if let Some(name) = item {
            let found = self.menu.iter().find(|m| m.name.to_lowercase().contains(&name.to_lowercase()));
            if let Some(m) = found {
                return format!("item_price:{}|{}", m.name, m.price_euros);
            }
            return format!("item_not_found:{}", name);
        }
        "price_general:our prices range from €6 to €75".to_string()
    }

    fn get_takeaway_info(&self) -> String {
        "takeaway:yes|We offer takeaway and delivery. Order by phone or at the counter.".to_string()
    }

    fn get_event_info(&self, location: Option<&str>) -> String {
        let event_spaces = ["terrace", "private room"];
        if let Some(loc) = location {
            let available = event_spaces.iter().any(|s| loc.to_lowercase().contains(s));
            if available {
                return format!("event_space_available:{}|Contact us at events@example.com to book.", loc);
            }
            return format!("event_space_unavailable:{}|We have a terrace and a private room available for events.", loc);
        }
        "event_info:We host birthday parties, corporate events, and private dinners. Spaces: terrace and private room. Contact events@example.com.".to_string()
    }

    fn get_facility_info(&self, facility: Option<&str>) -> String {
        if let Some(f) = facility {
            let available = self.facilities.iter().any(|fac| fac.to_lowercase().contains(&f.to_lowercase()));
            if available {
                return format!("facility_available:{}", f);
            }
            return format!("facility_unavailable:{}", f);
        }
        format!("all_facilities:{}", self.facilities.join(", "))
    }

    fn get_accessibility_info(&self) -> String {
        "accessibility:yes|The restaurant is wheelchair accessible with step-free access at the main entrance. Strollers are welcome.".to_string()
    }

    fn get_entertainment_info(&self) -> String {
        "entertainment:yes|We have live music every Friday and Saturday evening. A DJ performs on Saturday nights.".to_string()
    }

    fn check_reservation(&self, reference: Option<&str>) -> String {
        if let Some(r) = reference {
            let found = self.reservations.iter().find(|res| res.reference.to_lowercase() == r.to_lowercase());
            if let Some(res) = found {
                return format!("found:{}|{}|{}|{}", res.reference, res.name, res.date, res.people_count);
            }
            return format!("not_found:{}", r);
        }
        "no_reference:".to_string()
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
        let result = adapter().check_reservation(Some("REST-ABC123"));
        assert!(result.starts_with("found:"));
        assert!(result.contains("Maya Chen"));
    }

    #[test]
    fn check_reservation_unknown_reference_returns_not_found() {
        let result = adapter().check_reservation(Some("REST-UNKNOWN"));
        assert!(result.starts_with("not_found:"));
    }

    #[test]
    fn check_reservation_no_reference_returns_no_reference() {
        assert!(adapter().check_reservation(None).starts_with("no_reference:"));
    }

    #[test]
    fn get_menu_dietary_vegan_returns_matching_items() {
        let result = adapter().get_menu_dietary(Some("vegan"));
        assert!(result.starts_with("dietary_results:"));
        assert!(result.contains("salad"));
        assert!(result.contains("vegan curry"));
    }

    #[test]
    fn get_menu_item_details_contains_allergen() {
        let result = adapter().get_menu_item_details(Some("pizza"), Some("gluten"));
        assert_eq!(result, "contains:pizza|gluten");
    }

    #[test]
    fn get_menu_item_details_not_contains_allergen() {
        let result = adapter().get_menu_item_details(Some("salad"), Some("gluten"));
        assert_eq!(result, "not_contains:salad|gluten");
    }

    #[test]
    fn get_payment_methods_accepted() {
        let result = adapter().get_payment_methods(Some("credit card"));
        assert!(result.starts_with("method_accepted:"));
    }

    #[test]
    fn get_payment_methods_not_accepted() {
        let result = adapter().get_payment_methods(Some("crypto"));
        assert!(result.starts_with("method_not_accepted:"));
    }

    #[test]
    fn get_facility_available() {
        let result = adapter().get_facility_info(Some("wifi"));
        assert_eq!(result, "facility_available:wifi");
    }

    #[test]
    fn get_facility_unavailable() {
        let result = adapter().get_facility_info(Some("swimming pool"));
        assert!(result.starts_with("facility_unavailable:"));
    }
}
