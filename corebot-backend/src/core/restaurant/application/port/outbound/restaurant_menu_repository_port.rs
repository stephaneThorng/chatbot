use uuid::Uuid;

use crate::core::restaurant::domain::model::{
    MenuItem, MenuPriceFilter, RestaurantRepositoryError,
};

#[async_trait::async_trait]
pub trait RestaurantMenuRepositoryPort {
    async fn menu_items(
        &self,
        business_id: Uuid,
        locale: &str,
    ) -> Result<Vec<MenuItem>, RestaurantRepositoryError>;
    async fn menu_items_by_price(
        &self,
        business_id: Uuid,
        locale: &str,
        filter: &MenuPriceFilter,
    ) -> Result<Vec<MenuItem>, RestaurantRepositoryError>;
}
