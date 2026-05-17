use std::sync::Arc;

use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;

#[derive(Clone)]
pub struct ConversationRestaurantMenuService<M> {
    pub(crate) menu_repository: M,
    pub(crate) business_info_repository:
        Arc<dyn RestaurantBusinessInfoRepositoryPort + Send + Sync>,
}

impl<M> ConversationRestaurantMenuService<M> {
    pub fn new(
        menu_repository: M,
        business_info_repository: Arc<dyn RestaurantBusinessInfoRepositoryPort + Send + Sync>,
    ) -> Self {
        Self {
            menu_repository,
            business_info_repository,
        }
    }
}
