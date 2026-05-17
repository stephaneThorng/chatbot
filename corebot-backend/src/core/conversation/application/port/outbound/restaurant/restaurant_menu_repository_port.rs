use std::sync::Arc;
use uuid::Uuid;

use crate::core::conversation::domain::restaurant::model::{
    AmountComparator, MenuItem, RestaurantRepositoryError,
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
        filter: &AmountComparator,
    ) -> Result<Vec<MenuItem>, RestaurantRepositoryError>;
}

#[async_trait::async_trait]
impl<T> RestaurantMenuRepositoryPort for Arc<T>
where
    T: RestaurantMenuRepositoryPort + Send + Sync + ?Sized,
{
    async fn menu_items(
        &self,
        business_id: Uuid,
        locale: &str,
    ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
        self.as_ref().menu_items(business_id, locale).await
    }

    async fn menu_items_by_price(
        &self,
        business_id: Uuid,
        locale: &str,
        filter: &AmountComparator,
    ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
        self.as_ref()
            .menu_items_by_price(business_id, locale, filter)
            .await
    }
}
