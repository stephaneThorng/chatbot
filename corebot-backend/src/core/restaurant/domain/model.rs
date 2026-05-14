/// In-memory domain model for the restaurant domain (v1).

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
    pub date: String,
    pub time: String,
    pub people_count: u32,
}

impl Reservation {
    pub fn new(reference: &str, name: &str, date: &str, time: &str, people_count: u32) -> Self {
        Self {
            reference: reference.to_string(),
            name: name.to_string(),
            date: date.to_string(),
            time: time.to_string(),
            people_count,
        }
    }
}
