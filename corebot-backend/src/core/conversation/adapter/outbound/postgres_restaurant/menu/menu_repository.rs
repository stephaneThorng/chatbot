use sqlx::PgPool;
use uuid::Uuid;

use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::conversation::domain::restaurant::model::{
    MenuItem, MenuPriceFilter, RestaurantRepositoryError,
};

use super::models::MenuItemRow;
use super::query_helpers::{hydrate_menu_items, parse_price_amount, repository_error};

#[derive(Clone)]
pub struct PostgresMenuRepository {
    pool: PgPool,
}

impl PostgresMenuRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RestaurantMenuRepositoryPort for PostgresMenuRepository {
    async fn menu_items(
        &self,
        business_id: Uuid,
        locale: &str,
    ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
        let rows = sqlx::query_as::<_, MenuItemRow>(
            r#"
            select mi.id, coalesce(requested.name, fallback.name) as name,
                   mi.price_cents, mi.currency
            from menu_items mi
            left join menu_item_translations requested
                on requested.menu_item_id = mi.id and requested.locale = $2
            left join menu_item_translations fallback
                on fallback.menu_item_id = mi.id and fallback.locale = 'en'
            where mi.business_id = $1 and mi.active
            order by coalesce(requested.name, fallback.name)
            "#,
        )
        .bind(business_id)
        .bind(locale)
        .fetch_all(&self.pool)
        .await
        .map_err(repository_error)?;

        hydrate_menu_items(&self.pool, rows).await
    }

    async fn menu_items_by_price(
        &self,
        business_id: Uuid,
        locale: &str,
        filter: &MenuPriceFilter,
    ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
        let Some(threshold) = parse_price_amount(&filter.amount) else {
            return Ok(vec![]);
        };

        let comparator = filter.comparator.to_lowercase();
        let operator = if matches!(comparator.as_str(), "under" | "less than" | "below")
            || comparator.starts_with("di bawah")
            || comparator.starts_with("kurang")
        {
            "<"
        } else if matches!(comparator.as_str(), "greater than" | "more than" | "over")
            || comparator.starts_with("lebih")
            || comparator.starts_with("di atas")
        {
            ">"
        } else {
            return Ok(vec![]);
        };

        let sql = format!(
            r#"
            select mi.id, coalesce(requested.name, fallback.name) as name,
                   mi.price_cents, mi.currency
            from menu_items mi
            left join menu_item_translations requested
                on requested.menu_item_id = mi.id and requested.locale = $2
            left join menu_item_translations fallback
                on fallback.menu_item_id = mi.id and fallback.locale = 'en'
            where mi.business_id = $1 and mi.active and mi.price_cents {operator} $3
            order by mi.price_cents, coalesce(requested.name, fallback.name)
            "#
        );

        let rows = sqlx::query_as::<_, MenuItemRow>(&sql)
            .bind(business_id)
            .bind(locale)
            .bind(threshold * 100)
            .fetch_all(&self.pool)
            .await
            .map_err(repository_error)?;

        hydrate_menu_items(&self.pool, rows).await
    }
}
