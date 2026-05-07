use std::sync::Arc;
use crate::core::conversation::application::port::output::domain_gateway_trait::DomainGateway;
use crate::core::restaurant::application::port::input::restaurant_trait::RestaurantPort;
/// Outbound adapter - bridges conversation and the restaurant domain.
/// Implements DomainGateway by delegating every call to RestaurantPort.
pub struct RestaurantDomainGateway {
    restaurant: Arc<dyn RestaurantPort>,
}
impl RestaurantDomainGateway {
    pub fn new(restaurant: Arc<dyn RestaurantPort>) -> Self {
        Self { restaurant }
    }
}
impl DomainGateway for RestaurantDomainGateway {
    fn get_opening_hours(&self) -> String {
        self.restaurant.get_opening_hours()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct StubRestaurantPort;
    impl RestaurantPort for StubRestaurantPort {
        fn get_opening_hours(&self) -> String {
            "Mon-Sun 9am-10pm".to_string()
        }
    }
    #[test]
    fn delegates_opening_hours_to_restaurant_port() {
        let gateway = RestaurantDomainGateway::new(Arc::new(StubRestaurantPort));
        assert_eq!(gateway.get_opening_hours(), "Mon-Sun 9am-10pm");
    }
}