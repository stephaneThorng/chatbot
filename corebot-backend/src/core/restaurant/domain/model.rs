/// In-memory domain model for the restaurant domain (v1).
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub name: String,
    pub dietary: Vec<String>,
    pub allergens: Vec<String>,
    pub price_euros: u32,
}

impl MenuItem {
    pub fn new(name: &str, dietary: &[&str], allergens: &[&str], price_euros: u32) -> Self {
        Self {
            name: name.to_string(),
            dietary: dietary.iter().map(|s| s.to_string()).collect(),
            allergens: allergens.iter().map(|s| s.to_string()).collect(),
            price_euros,
        }
    }

    pub fn has_dietary(&self, requirement: &str) -> bool {
        let req = requirement.to_lowercase();
        self.dietary.iter().any(|d| d.to_lowercase().contains(&req))
    }

    pub fn has_allergen(&self, allergen: &str) -> bool {
        let a = allergen.to_lowercase();
        self.allergens
            .iter()
            .any(|al| al.to_lowercase().contains(&a))
    }
}

#[derive(Debug, Clone)]
pub struct Reservation {
    pub reference: String,
    pub name: String,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub people_count: u32,
}

impl Reservation {
    pub fn new(
        reference: &str,
        name: &str,
        date: NaiveDate,
        time: NaiveTime,
        people_count: u32,
    ) -> Self {
        Self {
            reference: reference.to_string(),
            name: name.to_string(),
            date,
            time,
            people_count,
        }
    }
}

/// One type of table available in the restaurant.
#[derive(Debug, Clone)]
pub struct TableConfig {
    /// Seats at this table.
    pub capacity: u32,
    /// How many tables of this type exist.
    pub count: u32,
}

/// Static configuration for the restaurant (v1: hardcoded).
#[derive(Debug, Clone)]
pub struct RestaurantConfig {
    pub opening: NaiveTime,
    pub closing: NaiveTime,
    /// Duration of one reservation slot in minutes.
    pub slot_mins: u32,
    pub tables: Vec<TableConfig>,
}

impl RestaurantConfig {
    /// Default v1 configuration: open 11:00–22:00, 2-hour slots,
    /// 2×6-seat, 3×4-seat, 3×2-seat tables.
    pub fn default_v1() -> Self {
        Self {
            opening: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            closing: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            slot_mins: 120,
            tables: vec![
                TableConfig {
                    capacity: 6,
                    count: 2,
                },
                TableConfig {
                    capacity: 4,
                    count: 3,
                },
                TableConfig {
                    capacity: 2,
                    count: 3,
                },
            ],
        }
    }

    /// Total seating capacity across all tables.
    pub fn total_capacity(&self) -> u32 {
        self.tables.iter().map(|t| t.capacity * t.count).sum()
    }
}

/// Error returned when a reservation cannot be created.
#[derive(Debug, Clone, PartialEq)]
pub enum ReservationError {
    /// The requested time is outside opening hours.
    RestaurantClosed,
    /// No table combination can seat the party at the requested slot.
    /// The optional field carries the next available slot if one exists
    /// within the same day or the next few days.
    NoAvailability { next_slot: Option<NaiveDateTime> },
}
