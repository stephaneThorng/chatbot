/// Inbound port of the restaurant domain.
/// The restaurant adapter implements this trait to expose domain data.
pub trait RestaurantPort {
    fn get_opening_hours(&self) -> String;
}
