use std::fmt;

use crate::core::conversation::domain::model::slot::SlotDefinition;

/// Backend-owned identifier for intents the conversation core knows how to route.
///
/// The NLU model still returns string labels, so `Unknown` preserves forward
/// compatibility with artifacts that contain labels not yet handled by Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntentId {
    ReservationCreate,
    ReservationCancel,
    AskMenuGeneral,
    AskMenuDietary,
    AskMenuItemDetails,
    AskLocation,
    AskContact,
    AskOpeningHours,
    AskPaymentMethods,
    AskPrice,
    AskTakeawayDelivery,
    AskEvent,
    AskFacilities,
    AskAccessibility,
    AskEntertainment,
    CheckReservation,
    Greeting,
    Thanks,
    Goodbye,
    Affirmative,
    Negative,
    Cancel,
    Unknown(String),
}

impl IntentId {
    pub fn from(id: &str) -> Self {
        match id {
            "reservation_create" => Self::ReservationCreate,
            "reservation_cancel" => Self::ReservationCancel,
            "ask_menu_general" => Self::AskMenuGeneral,
            "ask_menu_dietary" => Self::AskMenuDietary,
            "ask_menu_item_details" => Self::AskMenuItemDetails,
            "ask_location" => Self::AskLocation,
            "ask_contact" => Self::AskContact,
            "ask_opening_hours" => Self::AskOpeningHours,
            "ask_payment_methods" => Self::AskPaymentMethods,
            "ask_price" => Self::AskPrice,
            "ask_takeaway_delivery" => Self::AskTakeawayDelivery,
            "ask_event" => Self::AskEvent,
            "ask_facilities" => Self::AskFacilities,
            "ask_accessibility" => Self::AskAccessibility,
            "ask_entertainment" => Self::AskEntertainment,
            "check_reservation" => Self::CheckReservation,
            "greeting" => Self::Greeting,
            "thanks" => Self::Thanks,
            "goodbye" => Self::Goodbye,
            "affirmative" => Self::Affirmative,
            "negative" => Self::Negative,
            "cancel" => Self::Cancel,
            value => Self::Unknown(value.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::ReservationCreate => "reservation_create",
            Self::ReservationCancel => "reservation_cancel",
            Self::AskMenuGeneral => "ask_menu_general",
            Self::AskMenuDietary => "ask_menu_dietary",
            Self::AskMenuItemDetails => "ask_menu_item_details",
            Self::AskLocation => "ask_location",
            Self::AskContact => "ask_contact",
            Self::AskOpeningHours => "ask_opening_hours",
            Self::AskPaymentMethods => "ask_payment_methods",
            Self::AskPrice => "ask_price",
            Self::AskTakeawayDelivery => "ask_takeaway_delivery",
            Self::AskEvent => "ask_event",
            Self::AskFacilities => "ask_facilities",
            Self::AskAccessibility => "ask_accessibility",
            Self::AskEntertainment => "ask_entertainment",
            Self::CheckReservation => "check_reservation",
            Self::Greeting => "greeting",
            Self::Thanks => "thanks",
            Self::Goodbye => "goodbye",
            Self::Affirmative => "affirmative",
            Self::Negative => "negative",
            Self::Cancel => "cancel",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl fmt::Display for IntentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentKind {
    Workflow,
    Informational,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct I18nKey(pub String);

impl I18nKey {
    pub fn new(key: &str) -> Self {
        Self(key.to_string())
    }
}

/// Handler-owned policy used by the conversation core to route and process an intent.
#[derive(Debug, PartialEq)]
pub struct IntentPolicy {
    pub id: IntentId,
    pub kind: IntentKind,
    pub nlu_task: Option<NluTask>,
    pub workflow_slots: Vec<SlotDefinition>,
    pub starting_message: Option<I18nKey>,
    pub confirmation_prompt: Option<I18nKey>,
    pub completion_response: Option<I18nKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NluTask {
    ReservationCreate,
    ReservationCancel,
    Choice,
}

impl NluTask {
    pub fn as_tag(&self) -> &'static str {
        match self {
            NluTask::ReservationCreate => "WF_RESERVATION_CREATE",
            NluTask::ReservationCancel => "WF_RESERVATION_CANCEL",
            NluTask::Choice => "WF_CHOICE",
        }
    }
}

pub fn i18n_key(key: &str) -> I18nKey {
    I18nKey::new(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_intent_label_maps_to_typed_variant() {
        assert_eq!(
            IntentId::from("ask_opening_hours"),
            IntentId::AskOpeningHours
        );
    }

    #[test]
    fn unknown_intent_label_is_preserved() {
        assert_eq!(
            IntentId::from("new_model_label"),
            IntentId::Unknown("new_model_label".to_string())
        );
        assert_eq!(
            IntentId::from("new_model_label").as_str(),
            "new_model_label"
        );
    }
}
