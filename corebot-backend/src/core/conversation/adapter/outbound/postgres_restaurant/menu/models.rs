use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub(crate) struct MenuItemRow {
    pub id: Uuid,
    pub name: String,
    pub price_cents: i32,
    pub currency: String,
}

#[derive(FromRow)]
pub(crate) struct TagRow {
    pub menu_item_id: Uuid,
    pub code: String,
}

#[derive(FromRow)]
pub(crate) struct IngredientRow {
    pub menu_item_id: Uuid,
    pub code: String,
}
