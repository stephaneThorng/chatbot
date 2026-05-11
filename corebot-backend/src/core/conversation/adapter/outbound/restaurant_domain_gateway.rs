use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::restaurant::application::port::inbound::restaurant_trait::RestaurantPort;

/// Outbound adapter - bridges conversation and the restaurant domain.
/// Implements DomainGateway by delegating every call to RestaurantPort.
pub struct RestaurantDomainGateway<R: RestaurantPort> {
    restaurant: R,
}

impl<R: RestaurantPort> RestaurantDomainGateway<R> {
    pub fn new(restaurant: R) -> Self {
        Self { restaurant }
    }
}

impl<R: RestaurantPort> DomainGatewayPort for RestaurantDomainGateway<R> {
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
        let gateway = RestaurantDomainGateway::new(StubRestaurantPort);
        assert_eq!(gateway.get_opening_hours(), "Mon-Sun 9am-10pm");
    }
}
