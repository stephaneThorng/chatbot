use sqlx::PgPool;
use uuid::Uuid;

use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::conversation::domain::restaurant::model::{
    AmountComparator, MenuItem, RestaurantRepositoryError,
};

use super::models::MenuItemRow;
use super::query_helpers::{hydrate_menu_items, repository_error};

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
        filter: &AmountComparator,
    ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
        let rows = match filter {
            AmountComparator::Equal(amount) => self.fetch_by_equal(business_id, locale, *amount).await?,
            AmountComparator::Above(amount) => self.fetch_by_ordering(business_id, locale, ">", *amount).await?,
            AmountComparator::Under(amount) => self.fetch_by_ordering(business_id, locale, "<", *amount).await?,
            AmountComparator::AtLeast(amount) => self.fetch_by_ordering(business_id, locale, ">=", *amount).await?,
            AmountComparator::AtMost(amount) => self.fetch_by_ordering(business_id, locale, "<=", *amount).await?,
            AmountComparator::Between(min, max) => {
                self.fetch_by_between(business_id, locale, *min, *max).await?
            }
        };

        hydrate_menu_items(&self.pool, rows).await
    }
}

impl PostgresMenuRepository {
    async fn fetch_by_ordering(
        &self,
        business_id: Uuid,
        locale: &str,
        operator: &str,
        amount: i32,
    ) -> Result<Vec<MenuItemRow>, RestaurantRepositoryError> {
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

        sqlx::query_as::<_, MenuItemRow>(&sql)
            .bind(business_id)
            .bind(locale)
            .bind(amount * 100)
            .fetch_all(&self.pool)
            .await
            .map_err(repository_error)
    }

    async fn fetch_by_equal(
        &self,
        business_id: Uuid,
        locale: &str,
        amount: i32,
    ) -> Result<Vec<MenuItemRow>, RestaurantRepositoryError> {
        sqlx::query_as::<_, MenuItemRow>(
            r#"
            select mi.id, coalesce(requested.name, fallback.name) as name,
                   mi.price_cents, mi.currency
            from menu_items mi
            left join menu_item_translations requested
                on requested.menu_item_id = mi.id and requested.locale = $2
            left join menu_item_translations fallback
                on fallback.menu_item_id = mi.id and fallback.locale = 'en'
            where mi.business_id = $1 and mi.active and mi.price_cents = $3
            order by mi.price_cents, coalesce(requested.name, fallback.name)
            "#,
        )
        .bind(business_id)
        .bind(locale)
        .bind(amount * 100)
        .fetch_all(&self.pool)
        .await
        .map_err(repository_error)
    }

    async fn fetch_by_between(
        &self,
        business_id: Uuid,
        locale: &str,
        min: i32,
        max: i32,
    ) -> Result<Vec<MenuItemRow>, RestaurantRepositoryError> {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        sqlx::query_as::<_, MenuItemRow>(
            r#"
            select mi.id, coalesce(requested.name, fallback.name) as name,
                   mi.price_cents, mi.currency
            from menu_items mi
            left join menu_item_translations requested
                on requested.menu_item_id = mi.id and requested.locale = $2
            left join menu_item_translations fallback
                on fallback.menu_item_id = mi.id and fallback.locale = 'en'
            where mi.business_id = $1 and mi.active and mi.price_cents between $3 and $4
            order by mi.price_cents, coalesce(requested.name, fallback.name)
            "#,
        )
        .bind(business_id)
        .bind(locale)
        .bind(min * 100)
        .bind(max * 100)
        .fetch_all(&self.pool)
        .await
        .map_err(repository_error)
    }
}
