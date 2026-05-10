/// Entity labels known by the conversation core.
///
/// `Unknown` is only used at the NLU boundary so handlers and workflow logic can
/// avoid string comparisons for supported entity labels.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    Person,
    Date,
    Time,
    PeopleCount,
    ReservationReference,
    MenuItem,
    Allergen,
    DietaryRequirement,
    Unknown(String),
}

impl EntityType {
    pub fn from(label: &str) -> Self {
        match label {
            "person" => Self::Person,
            "date" => Self::Date,
            "time" => Self::Time,
            "people_count" => Self::PeopleCount,
            "reservation_reference" => Self::ReservationReference,
            "menu_item" => Self::MenuItem,
            "allergen" => Self::Allergen,
            "dietary_requirement" => Self::DietaryRequirement,
            value => Self::Unknown(value.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_entity_label_maps_to_typed_variant() {
        assert_eq!(EntityType::from("menu_item"), EntityType::MenuItem);
    }

    #[test]
    fn unknown_entity_label_is_preserved() {
        assert_eq!(
            EntityType::from("dish_name"),
            EntityType::Unknown("dish_name".to_string())
        );
    }
}
