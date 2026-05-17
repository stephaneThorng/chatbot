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
    items.iter()
        .filter(|item| contains_text(&item.ingredients, requested))
        .collect()
}

pub(super) fn format_items(items: &[MenuItem]) -> String {
    items.iter()
        .map(|item| format!("{} (EUR {})", item.name, price_euros(item.price_cents)))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn format_item_refs(items: &[&MenuItem]) -> String {
    items.iter()
        .map(|item| format!("{} (EUR {})", item.name, price_euros(item.price_cents)))
        .collect::<Vec<_>>()
        .join(", ")
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

pub(super) fn price_euros(price_cents: i32) -> i32 {
    price_cents / 100
}

#[cfg(test)]
mod tests {
    use super::{canonical_tokens, contains_text};

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
}
