use std::collections::HashMap;

use super::domain_type::DomainType;
use super::slot::SlotType;

/// Intent identifier - a string, not an enum.
/// New intents can be added via config/DB without code changes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntentId(pub String);

impl IntentId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

/// The two kinds of intent behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentKind {
    /// Multi-turn: Idle -> Workflow(slots) -> confirmation -> execute -> Idle
    Workflow,
    /// Single-turn: Idle -> answer (optionally with slots) -> Idle
    Informational,
}

/// Metadata describing how an intent behaves in conversation.
/// Loaded at startup from config or database.
#[derive(Debug, Clone)]
pub struct IntentPolicy {
    pub id: IntentId,
    pub domain: DomainType,
    pub kind: IntentKind,
    /// The NLU task context when this intent's workflow is active.
    pub nlu_task: Option<NluTask>,
    /// Required slots. Empty means no data collection needed.
    /// For workflows: confirmation is always appended automatically.
    pub required_slots: Vec<SlotDefinition>,
}

/// NLU task context tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NluTask {
    Open,
    BookRequirement,
}

impl NluTask {
    pub fn as_tag(&self) -> &'static str {
        match self {
            NluTask::Open => "open",
            NluTask::BookRequirement => "book_requirement",
        }
    }
}

/// Definition of a slot.
#[derive(Debug, Clone)]
pub struct SlotDefinition {
    pub name: String,
    pub slot_type: SlotType,
    pub required: bool,
}

/// The catalog of all known intents and their policies.
/// Built once at startup, immutable at runtime.
pub struct IntentCatalog {
    policies: HashMap<IntentId, IntentPolicy>,
}

impl IntentCatalog {
    pub fn new(policies: Vec<IntentPolicy>) -> Self {
        let map = policies.into_iter().map(|p| (p.id.clone(), p)).collect();
        Self { policies: map }
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

    pub fn nlu_task(&self, id: &IntentId) -> NluTask {
        self.policies
            .get(id)
            .and_then(|p| p.nlu_task)
            .unwrap_or(NluTask::Open)
    }

    pub fn for_domain(&self, domain: DomainType) -> Vec<&IntentPolicy> {
        self.policies
            .values()
            .filter(|p| p.domain == domain)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Default catalog builders
// ---------------------------------------------------------------------------

pub fn build_restaurant_catalog() -> IntentCatalog {
    IntentCatalog::new(vec![
        // -- Workflow intents --
        workflow(
            "book",
            DomainType::Restaurant,
            Some(NluTask::BookRequirement),
            vec![
                slot("name", SlotType::Text),
                slot("date", SlotType::Date),
                slot("time", SlotType::Time),
                slot("people", SlotType::Number),
            ],
        ),
        workflow(
            "cancel",
            DomainType::Restaurant,
            Some(NluTask::BookRequirement),
            vec![],
        ),
        // -- Informational intents (some with optional slots) --
        informational(
            "menu",
            DomainType::Restaurant,
            vec![SlotDefinition {
                name: "dietary".into(),
                slot_type: SlotType::Text,
                required: false,
            }],
        ),
        informational("location", DomainType::Restaurant, vec![]),
        informational("contact", DomainType::Restaurant, vec![]),
        informational("opening_hours", DomainType::Restaurant, vec![]),
        // -- Conversational intents --
        informational("greeting", DomainType::Restaurant, vec![]),
        informational("thanks", DomainType::Restaurant, vec![]),
        informational("farewell", DomainType::Restaurant, vec![]),
        // -- Requirement signals (NLU returns during [TASK=book_requirement]) --
        informational("affirmative", DomainType::Restaurant, vec![]),
        informational("negative", DomainType::Restaurant, vec![]),
        informational("provide_info", DomainType::Restaurant, vec![]),
        // -- Unknown --
        informational("unknown", DomainType::Restaurant, vec![]),
    ])
}

fn workflow(
    id: &str,
    domain: DomainType,
    nlu_task: Option<NluTask>,
    slots: Vec<SlotDefinition>,
) -> IntentPolicy {
    IntentPolicy {
        id: IntentId::new(id),
        domain,
        kind: IntentKind::Workflow,
        nlu_task,
        required_slots: slots,
    }
}

fn informational(id: &str, domain: DomainType, slots: Vec<SlotDefinition>) -> IntentPolicy {
    IntentPolicy {
        id: IntentId::new(id),
        domain,
        kind: IntentKind::Informational,
        nlu_task: None,
        required_slots: slots,
    }
}

fn slot(name: &str, slot_type: SlotType) -> SlotDefinition {
    SlotDefinition {
        name: name.into(),
        slot_type,
        required: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> IntentCatalog {
        build_restaurant_catalog()
    }

    #[test]
    fn book_is_workflow() {
        assert!(catalog().is_workflow(&IntentId::new("book")));
    }

    #[test]
    fn menu_is_informational() {
        assert!(catalog().is_informational(&IntentId::new("menu")));
    }

    #[test]
    fn menu_has_optional_dietary_slot() {
        let slots = catalog().required_slots(&IntentId::new("menu"));
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].name, "dietary");
        assert!(!slots[0].required);
    }

    #[test]
    fn book_has_4_data_slots() {
        assert_eq!(catalog().required_slots(&IntentId::new("book")).len(), 4);
    }

    #[test]
    fn cancel_has_no_data_slots() {
        assert_eq!(catalog().required_slots(&IntentId::new("cancel")).len(), 0);
    }
}
