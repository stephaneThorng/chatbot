use crate::core::restaurant::application::port::input::restaurant_trait::RestaurantPort;
/// Stub implementation of RestaurantPort.
/// Replace each method body with real data sources when available.
pub struct RestaurantAdapter;
impl RestaurantPort for RestaurantAdapter {
    fn get_opening_hours(&self) -> String {
        "Not implemented yet".to_string()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_opening_hours_returns_stub() {
        assert_eq!(RestaurantAdapter.get_opening_hours(), "Not implemented yet");
    }
}
