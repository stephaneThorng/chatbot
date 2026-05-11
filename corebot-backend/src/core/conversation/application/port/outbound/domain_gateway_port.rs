/// Outbound port - what the conversation use case needs from a domain.
/// Each domain (restaurant, hotel) provides its own implementation.
pub trait DomainGatewayPort {
    fn get_opening_hours(&self) -> String;
}
