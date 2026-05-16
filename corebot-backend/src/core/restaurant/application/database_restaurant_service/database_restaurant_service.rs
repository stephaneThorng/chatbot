use uuid::Uuid;

#[derive(Clone)]
pub struct DatabaseRestaurantService<B, M, R, A> {
    pub(crate) business_id: Uuid,
    pub(crate) locale: String,
    pub(crate) business_info_repository: B,
    pub(crate) menu_repository: M,
    pub(crate) reservation_repository: R,
    pub(crate) availability_repository: A,
}

impl<B, M, R, A> DatabaseRestaurantService<B, M, R, A> {
    pub fn new(
        business_id: Uuid,
        locale: impl Into<String>,
        business_info_repository: B,
        menu_repository: M,
        reservation_repository: R,
        availability_repository: A,
    ) -> Self {
        Self {
            business_id,
            locale: locale.into(),
            business_info_repository,
            menu_repository,
            reservation_repository,
            availability_repository,
        }
    }
}
