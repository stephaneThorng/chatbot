use crate::core::conversation::domain::restaurant::model::MenuItem;

pub(super) fn find_item<'a>(items: &'a [MenuItem], item_name: &str) -> Option<&'a MenuItem> {
    let requested_tokens = tokenize(item_name);
    let requested_text = normalized_text(item_name);

    items.iter().find(|item| {
        let item_text = normalized_text(&item.name);
        let item_tokens = tokenize(&item.name);

        !requested_tokens.is_empty()
            && (item_tokens.starts_with(&requested_tokens)
                || requested_tokens
                    .iter()
                    .all(|token| item_tokens.iter().any(|item_token| item_token == token))
                || item_text.contains(&requested_text))
    })
}

pub(super) fn find_exact_item<'a>(items: &'a [MenuItem], item_name: &str) -> Option<&'a MenuItem> {
    let requested_tokens = canonical_tokens(item_name);
    items.iter().find(|item| canonical_tokens(&item.name) == requested_tokens)
}

pub(super) fn contains_text(values: &[String], requested: &str) -> bool {
    let requested_tokens = canonical_tokens(requested);
    values
        .iter()
        .any(|value| canonical_tokens(value) == requested_tokens)
}

pub(super) fn filter_by_ingredient<'a>(
    items: &'a [MenuItem],
    requested: &str,
) -> Vec<&'a MenuItem> {
    let aliases = ingredient_aliases(requested);
    items.iter()
        .filter(|item| {
            contains_text(&item.ingredients, requested)
                || aliases
                    .iter()
                    .any(|alias| contains_text(&item.ingredients, alias))
        })
        .collect()
}

pub(super) fn filter_without_ingredient<'a>(
    items: &'a [MenuItem],
    requested: &str,
) -> Vec<&'a MenuItem> {
    let aliases = ingredient_aliases(requested);
    items.iter()
        .filter(|item| {
            !contains_text(&item.ingredients, requested)
                && !aliases
                    .iter()
                    .any(|alias| contains_text(&item.ingredients, alias))
        })
        .collect()
}

pub(super) fn matches_dietary_requirement(item: &MenuItem, requested: &str) -> bool {
    contains_text(&item.dietary, requested)
        || derived_allergen_aliases(requested)
            .iter()
            .all(|alias| !contains_text(&item.allergens, alias))
}

pub(super) fn format_items(items: &[MenuItem]) -> String {
    bullet_list(items.iter().map(format_item))
}

pub(super) fn format_item_refs(items: &[&MenuItem]) -> String {
    bullet_list(items.iter().map(|item| format_item(item)))
}

pub(super) fn format_item(item: &MenuItem) -> String {
    format!("{} ({})", item.name, format_price(item.price_cents, &item.currency))
}

pub(super) fn format_price(price_cents: i32, currency: &str) -> String {
    let amount = price_cents / 100;
    match currency {
        "EUR" => format!("EUR {}", amount),
        "USD" => format!("USD {}", amount),
        "IDR" => format!("IDR {}", with_thousands_separators(amount)),
        _ => format!("{} {}", currency, amount),
    }
}

pub(super) fn canonical_tokens(text: &str) -> Vec<String> {
    let mut tokens = tokenize(text)
        .into_iter()
        .map(|token| {
            let mut chars = token.chars().collect::<Vec<_>>();
            chars.sort_unstable();
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<_>>();
    tokens.sort();
    tokens.dedup();
    tokens
}

fn normalized_text(text: &str) -> String {
    tokenize(text).join(" ")
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_lowercase())
        .collect()
}

fn derived_allergen_aliases(requested: &str) -> Vec<String> {
    let normalized = normalized_text(requested);
    if normalized.contains("nut free") || normalized.contains("nuts free") {
        return vec!["peanuts".to_string(), "tree nuts".to_string()];
    }
    if let Some(base) = normalized
        .strip_suffix(" free")
        .or_else(|| normalized.strip_suffix("free"))
    {
        let trimmed = base.trim();
        if !trimmed.is_empty() {
            return vec![match trimmed {
                "egg" => "eggs".to_string(),
                "nut" | "nuts" => "peanuts".to_string(),
                value => value.to_string(),
            }];
        }
    }
    vec![]
}

fn with_thousands_separators(value: i32) -> String {
    let digits = value.abs().to_string();
    let grouped = digits
        .chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(",");
    let formatted = grouped.chars().rev().collect::<String>();
    if value < 0 {
        format!("-{formatted}")
    } else {
        formatted
    }
}

fn ingredient_aliases(requested: &str) -> Vec<&'static str> {
    match normalized_text(requested).as_str() {
        "alcohol" | "alcoholic drink" | "alcoholic drinks" => vec!["beer"],
        _ => vec![],
    }
}

fn bullet_list(values: impl Iterator<Item = String>) -> String {
    values
        .map(|value| format!("- {value}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{canonical_tokens, contains_text, format_price, matches_dietary_requirement};
    use crate::core::conversation::domain::restaurant::model::MenuItem;

    fn menu_item(dietary: &[&str], allergens: &[&str]) -> MenuItem {
        MenuItem {
            name: "sample".to_string(),
            ingredients: vec![],
            dietary: dietary.iter().map(|value| value.to_string()).collect(),
            allergens: allergens.iter().map(|value| value.to_string()).collect(),
            price_cents: 4_500_000,
            currency: "IDR".to_string(),
        }
    }

    #[test]
    fn canonical_tokens_ignore_separator_case_and_token_order() {
        assert_eq!(canonical_tokens("gluten-free"), canonical_tokens("gluten free"));
        assert_eq!(canonical_tokens("gluten-free"), canonical_tokens("Free_GlUten"));
    }

    #[test]
    fn canonical_tokens_accept_letter_permutation_per_token() {
        assert_eq!(canonical_tokens("gluten free"), canonical_tokens("Gluent_Free"));
    }

    #[test]
    fn contains_text_uses_canonical_token_matching() {
        assert!(contains_text(&["gluten-free".to_string()], "Free_GlUten"));
    }

    #[test]
    fn dietary_match_can_fall_back_to_allergen_exclusion() {
        assert!(matches_dietary_requirement(
            &menu_item(&[], &["gluten"]),
            "nut free"
        ));
        assert!(!matches_dietary_requirement(
            &menu_item(&[], &["tree nuts"]),
            "nut free"
        ));
    }

    #[test]
    fn format_price_supports_idr() {
        assert_eq!(format_price(4_500_000, "IDR"), "IDR 45,000");
    }
}
