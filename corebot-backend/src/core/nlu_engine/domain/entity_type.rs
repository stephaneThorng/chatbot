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
    PriceItem,
    Location,
    Phone,
    Email,
    Allergen,
    Facility,
    PaymentMethod,
    PriceComparator,
    PriceAmount,
    DietaryRequirement,
    Unknown(String),
}

impl EntityType {
    pub fn as_label(&self) -> &str {
        match self {
            Self::Person => "person",
            Self::Date => "date",
            Self::Time => "time",
            Self::PeopleCount => "people_count",
            Self::ReservationReference => "reservation_reference",
            Self::MenuItem => "menu_item",
            Self::PriceItem => "price_item",
            Self::Location => "location",
            Self::Phone => "phone",
            Self::Email => "email",
            Self::Allergen => "allergen",
            Self::Facility => "facility",
            Self::PaymentMethod => "payment_method",
            Self::PriceComparator => "price_comparator",
            Self::PriceAmount => "price_amount",
            Self::DietaryRequirement => "dietary_requirement",
            Self::Unknown(s) => s.as_str(),
        }
    }

    pub fn from(label: &str) -> Self {
        match label {
            "person" => Self::Person,
            "date" => Self::Date,
            "time" => Self::Time,
            "people_count" => Self::PeopleCount,
            "reservation_reference" => Self::ReservationReference,
            "menu_item" => Self::MenuItem,
            "price_item" => Self::PriceItem,
            "location" => Self::Location,
            "phone" => Self::Phone,
            "email" => Self::Email,
            "allergen" => Self::Allergen,
            "facility" => Self::Facility,
            "payment_method" => Self::PaymentMethod,
            "price_comparator" => Self::PriceComparator,
            "price_amount" => Self::PriceAmount,
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
    fn restaurant_dataset_entity_labels_map_to_typed_variants() {
        for label in [
            "person",
            "date",
            "time",
            "people_count",
            "reservation_reference",
            "menu_item",
            "price_item",
            "location",
            "phone",
            "email",
            "allergen",
            "facility",
            "payment_method",
            "price_comparator",
            "price_amount",
            "dietary_requirement",
        ] {
            assert!(
                !matches!(EntityType::from(label), EntityType::Unknown(_)),
                "{label} should map to a typed EntityType"
            );
        }
    }

    #[test]
    fn unknown_entity_label_is_preserved() {
        assert_eq!(
            EntityType::from("dish_name"),
            EntityType::Unknown("dish_name".to_string())
        );
    }
}
