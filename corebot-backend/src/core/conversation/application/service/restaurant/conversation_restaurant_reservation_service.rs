#[derive(Clone)]
pub struct ConversationRestaurantReservationService<R, A> {
    pub(crate) reservation_repository: R,
    pub(crate) availability_repository: A,
}

impl<R, A> ConversationRestaurantReservationService<R, A> {
    pub fn new(reservation_repository: R, availability_repository: A) -> Self {
        Self {
            reservation_repository,
            availability_repository,
        }
    }
}
