use sqlx::PgPool;
use uuid::Uuid;

use crate::core::conversation::domain::restaurant::model::{MenuItem, RestaurantRepositoryError};

use super::models::{IngredientRow, MenuItemRow, TagRow};

pub(crate) fn repository_error(error: sqlx::Error) -> RestaurantRepositoryError {
    RestaurantRepositoryError {
        message: error.to_string(),
    }
}

pub(crate) async fn hydrate_menu_items(
    pool: &PgPool,
    rows: Vec<MenuItemRow>,
) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
    if rows.is_empty() {
        return Ok(vec![]);
    }

    let ids = rows.iter().map(|row| row.id).collect::<Vec<Uuid>>();
    let dietary = sqlx::query_as::<_, TagRow>(
        r#"
        select midt.menu_item_id, dt.code
        from menu_item_dietary_tags midt
        join dietary_tags dt on dt.id = midt.dietary_tag_id
        where midt.menu_item_id = any($1)
        order by dt.code
        "#,
    )
    .bind(&ids)
    .fetch_all(pool)
    .await
    .map_err(repository_error)?;

    let allergens = sqlx::query_as::<_, TagRow>(
        r#"
        select miat.menu_item_id, at.code
        from menu_item_allergen_tags miat
        join allergen_tags at on at.id = miat.allergen_tag_id
        where miat.menu_item_id = any($1)
        order by at.code
        "#,
    )
    .bind(&ids)
    .fetch_all(pool)
    .await
    .map_err(repository_error)?;

    let ingredients = sqlx::query_as::<_, IngredientRow>(
        r#"
        select mii.menu_item_id, it.code
        from menu_item_ingredients mii
        join ingredient_tags it on it.id = mii.ingredient_tag_id
        where mii.menu_item_id = any($1)
        order by it.code
        "#,
    )
    .bind(&ids)
    .fetch_all(pool)
    .await
    .map_err(repository_error)?;

    Ok(rows
        .into_iter()
        .map(|row| MenuItem {
            ingredients: ingredients
                .iter()
                .filter(|tag| tag.menu_item_id == row.id)
                .map(|tag| tag.code.clone())
                .collect(),
            dietary: dietary
                .iter()
                .filter(|tag| tag.menu_item_id == row.id)
                .map(|tag| tag.code.clone())
                .collect(),
            allergens: allergens
                .iter()
                .filter(|tag| tag.menu_item_id == row.id)
                .map(|tag| tag.code.clone())
                .collect(),
            name: row.name,
            price_cents: row.price_cents,
            currency: row.currency,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::core::conversation::domain::restaurant::model::AmountComparator;

    #[test]
    fn amount_comparator_between_is_orderable() {
        let comparator = AmountComparator::Between(10, 20);
        assert_eq!(comparator, AmountComparator::Between(10, 20));
    }
}
