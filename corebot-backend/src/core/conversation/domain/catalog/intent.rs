use std::collections::HashMap;

use crate::core::conversation::domain::model::domain_type::DomainType;
use crate::core::conversation::domain::model::slot::SlotType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntentId(pub String);

impl IntentId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentKind {
    Workflow,
    Informational,
}

#[derive(Debug, Clone)]
pub struct I18nKey(pub String);

impl I18nKey {
    pub fn new(key: &str) -> Self {
        Self(key.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum IntentResponse {
    Static(I18nKey),
    DomainOpeningHours,
    EchoIntent,
}

#[derive(Debug, Clone)]
pub struct IntentPolicy {
    pub id: IntentId,
    pub domain: DomainType,
    pub kind: IntentKind,
    pub nlu_task: Option<NluTask>,
    pub required_slots: Vec<SlotDefinition>,
    pub response: IntentResponse,
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

#[derive(Debug, Clone)]
pub struct SlotDefinition {
    pub name: String,
    pub slot_type: SlotType,
    pub required: bool,
    pub entity_types: Vec<String>,
    pub prompt: I18nKey,
}

pub struct IntentCatalog {
    policies: HashMap<IntentId, IntentPolicy>,
    system_texts: HashMap<String, I18nKey>,
}

impl IntentCatalog {
    pub fn new(policies: Vec<IntentPolicy>, system_texts: HashMap<String, I18nKey>) -> Self {
        let map = policies.into_iter().map(|p| (p.id.clone(), p)).collect();
        Self {
            policies: map,
            system_texts,
        }
    }

    pub fn get(&self, id: &IntentId) -> Option<&IntentPolicy> {
        self.policies.get(id)
    }

    pub fn is_workflow(&self, id: &IntentId) -> bool {
        self.policies
            .get(id)
            .is_some_and(|p| p.kind == IntentKind::Workflow)
    }

    pub fn is_informational(&self, id: &IntentId) -> bool {
        self.policies
            .get(id)
            .is_some_and(|p| p.kind == IntentKind::Informational)
    }

    pub fn required_slots(&self, id: &IntentId) -> Vec<SlotDefinition> {
        self.policies
            .get(id)
            .map_or(vec![], |p| p.required_slots.clone())
    }

    pub fn nlu_task(&self, id: &IntentId) -> Option<NluTask> {
        self.policies.get(id).and_then(|p| p.nlu_task)
    }

    pub fn for_domain(&self, domain: DomainType) -> Vec<&IntentPolicy> {
        self.policies
            .values()
            .filter(|p| p.domain == domain)
            .collect()
    }

    pub fn system_text_key(&self, key: &str) -> Option<&str> {
        self.system_texts.get(key).map(|text| text.0.as_str())
    }

    pub fn slot_prompt_key(&self, intent: &IntentId, slot_name: &str) -> Option<&str> {
        self.policies
            .get(intent)?
            .required_slots
            .iter()
            .find(|slot| slot.name == slot_name)
            .map(|slot| slot.prompt.0.as_str())
    }

    pub fn confirmation_prompt_key(&self, intent: &IntentId) -> Option<&str> {
        self.policies
            .get(intent)?
            .confirmation_prompt
            .as_ref()
            .map(|text| text.0.as_str())
    }

    pub fn completion_response_key(&self, intent: &IntentId) -> Option<&str> {
        self.policies
            .get(intent)?
            .completion_response
            .as_ref()
            .map(|text| text.0.as_str())
    }
}

pub fn build_catalog(domain: DomainType) -> IntentCatalog {
    match domain {
        DomainType::Restaurant => build_restaurant_catalog(),
        DomainType::Hotel => build_hotel_catalog(),
    }
}

pub fn build_restaurant_catalog() -> IntentCatalog {
    IntentCatalog::new(
        vec![
            workflow(
                "reservation_create",
                DomainType::Restaurant,
                Some(NluTask::ReservationCreate),
                vec![
                    required_slot(
                        "name",
                        SlotType::Text,
                        &["person"],
                        i18n_key("workflow.reservation_create.slot.name.prompt"),
                    ),
                    required_slot(
                        "date",
                        SlotType::Date,
                        &["date"],
                        i18n_key("workflow.reservation_create.slot.date.prompt"),
                    ),
                    required_slot(
                        "time",
                        SlotType::Time,
                        &["time"],
                        i18n_key("workflow.reservation_create.slot.time.prompt"),
                    ),
                    required_slot(
                        "people",
                        SlotType::Number,
                        &["people_count"],
                        i18n_key("workflow.reservation_create.slot.people.prompt"),
                    ),
                ],
                IntentResponse::EchoIntent,
                Some(i18n_key("workflow.reservation_create.confirmation.prompt")),
                Some(i18n_key("workflow.reservation_create.completion.success")),
            ),
            workflow(
                "reservation_cancel",
                DomainType::Restaurant,
                Some(NluTask::ReservationCancel),
                vec![
                    required_slot(
                        "reference",
                        SlotType::Text,
                        &["reservation_reference"],
                        i18n_key("workflow.reservation_cancel.slot.reference.prompt"),
                    ),
                    optional_slot(
                        "name",
                        SlotType::Text,
                        &["person"],
                        i18n_key("workflow.reservation_cancel.slot.name.prompt"),
                    ),
                    optional_slot(
                        "date",
                        SlotType::Date,
                        &["date"],
                        i18n_key("workflow.reservation_cancel.slot.date.prompt"),
                    ),
                ],
                IntentResponse::EchoIntent,
                Some(i18n_key("workflow.reservation_cancel.confirmation.prompt")),
                Some(i18n_key("workflow.reservation_cancel.completion.success")),
            ),
            informational(
                "ask_menu_general",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "ask_menu_dietary",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "ask_menu_item_details",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "ask_location",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "ask_contact",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "ask_opening_hours",
                DomainType::Restaurant,
                IntentResponse::DomainOpeningHours,
            ),
            informational(
                "ask_payment_methods",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "ask_price",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "ask_takeaway_delivery",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "ask_event",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "ask_facilities",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "ask_accessibility",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "ask_entertainment",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "check_reservation",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "greeting",
                DomainType::Restaurant,
                IntentResponse::Static(i18n_key("intent.greeting.reply")),
            ),
            informational(
                "thanks",
                DomainType::Restaurant,
                IntentResponse::Static(i18n_key("intent.thanks.reply")),
            ),
            informational(
                "goodbye",
                DomainType::Restaurant,
                IntentResponse::Static(i18n_key("intent.goodbye.reply")),
            ),
            informational(
                "affirmative",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational(
                "negative",
                DomainType::Restaurant,
                IntentResponse::EchoIntent,
            ),
            informational("cancel", DomainType::Restaurant, IntentResponse::EchoIntent),
            informational(
                "unknown",
                DomainType::Restaurant,
                IntentResponse::Static(i18n_key("intent.unknown.reply")),
            ),
        ],
        restaurant_system_texts(),
    )
}

fn build_hotel_catalog() -> IntentCatalog {
    IntentCatalog::new(vec![], restaurant_system_texts())
}

fn restaurant_system_texts() -> HashMap<String, I18nKey> {
    [
        ("no_active_workflow", i18n_key("system.no_active_workflow")),
        (
            "no_active_workflow_to_cancel",
            i18n_key("system.no_active_workflow_to_cancel"),
        ),
        ("workflow_cancelled", i18n_key("system.workflow_cancelled")),
        ("confirm_yes_no", i18n_key("system.confirm_yes_no")),
        ("workflow_complete", i18n_key("system.workflow_complete")),
        ("echo_intent", i18n_key("system.echo_intent")),
        (
            "missing_slot_fallback",
            i18n_key("system.missing_slot_fallback"),
        ),
        ("confirm_generic", i18n_key("system.confirm_generic")),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value))
    .collect()
}

fn workflow(
    id: &str,
    domain: DomainType,
    nlu_task: Option<NluTask>,
    slots: Vec<SlotDefinition>,
    response: IntentResponse,
    confirmation_prompt: Option<I18nKey>,
    completion_response: Option<I18nKey>,
) -> IntentPolicy {
    IntentPolicy {
        id: IntentId::new(id),
        domain,
        kind: IntentKind::Workflow,
        nlu_task,
        required_slots: slots,
        response,
        confirmation_prompt,
        completion_response,
    }
}

fn informational(id: &str, domain: DomainType, response: IntentResponse) -> IntentPolicy {
    IntentPolicy {
        id: IntentId::new(id),
        domain,
        kind: IntentKind::Informational,
        nlu_task: None,
        required_slots: vec![],
        response,
        confirmation_prompt: None,
        completion_response: None,
    }
}

fn required_slot(
    name: &str,
    slot_type: SlotType,
    entity_types: &[&str],
    prompt: I18nKey,
) -> SlotDefinition {
    SlotDefinition {
        name: name.into(),
        slot_type,
        required: true,
        entity_types: entity_types
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        prompt,
    }
}

fn optional_slot(
    name: &str,
    slot_type: SlotType,
    entity_types: &[&str],
    prompt: I18nKey,
) -> SlotDefinition {
    SlotDefinition {
        name: name.into(),
        slot_type,
        required: false,
        entity_types: entity_types
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        prompt,
    }
}

fn i18n_key(key: &str) -> I18nKey {
    I18nKey::new(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> IntentCatalog {
        build_restaurant_catalog()
    }

    #[test]
    fn book_is_workflow() {
        assert!(catalog().is_workflow(&IntentId::new("reservation_create")));
    }

    #[test]
    fn menu_is_informational() {
        assert!(catalog().is_informational(&IntentId::new("ask_menu_general")));
    }

    #[test]
    fn reservation_cancel_has_reference_slot() {
        let slots = catalog().required_slots(&IntentId::new("reservation_cancel"));
        assert_eq!(slots[0].name, "reference");
        assert!(slots[0].required);
    }

    #[test]
    fn book_has_4_data_slots() {
        assert_eq!(
            catalog()
                .required_slots(&IntentId::new("reservation_create"))
                .len(),
            4
        );
    }

    #[test]
    fn reservation_create_has_indonesian_confirmation_prompt() {
        let catalog = catalog();
        let key = catalog
            .confirmation_prompt_key(&IntentId::new("reservation_create"))
            .unwrap();
        assert_eq!(key, "workflow.reservation_create.confirmation.prompt");
    }
}
