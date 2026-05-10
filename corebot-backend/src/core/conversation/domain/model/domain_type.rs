/// The business domain this session belongs to.
/// Fixed per session (determined by access token or endpoint).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainType {
    Restaurant,
    Hotel,
}

impl DomainType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DomainType::Restaurant => "restaurant",
            DomainType::Hotel => "hotel",
        }
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restaurant_as_str() {
        assert_eq!(DomainType::Restaurant.as_str(), "restaurant");
    }

    #[test]
    fn hotel_as_str() {
        assert_eq!(DomainType::Hotel.as_str(), "hotel");
    }
}
