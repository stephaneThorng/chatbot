use crate::core::restaurant::domain::model::MenuItem;

pub(super) fn find_item<'a>(items: &'a [MenuItem], item_name: &str) -> Option<&'a MenuItem> {
    let normalized = item_name.to_lowercase();
    items
        .iter()
        .find(|item| item.name.to_lowercase().contains(&normalized))
}

pub(super) fn contains_text(values: &[String], requested: &str) -> bool {
    let normalized = requested.to_lowercase();
    values
        .iter()
        .any(|value| value.to_lowercase().contains(&normalized))
}

pub(super) fn format_items(items: &[MenuItem]) -> String {
    items
        .iter()
        .map(|item| format!("{} (EUR {})", item.name, price_euros(item.price_cents)))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn price_euros(price_cents: i32) -> i32 {
    price_cents / 100
}
