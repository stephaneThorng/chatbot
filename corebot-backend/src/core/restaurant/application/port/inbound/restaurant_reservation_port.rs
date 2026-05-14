use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    ReservationCreateQuery, ReservationLookupQuery,
};
use crate::core::restaurant::domain::model::ReservationError;

pub trait RestaurantReservationUseCase {
    fn create_reservation(&self, query: ReservationCreateQuery) -> Result<String, ReservationError>;
    fn check_reservation(&self, query: ReservationLookupQuery) -> String;
}
