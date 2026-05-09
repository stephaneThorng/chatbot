use super::conversation_id::ConversationId;
use super::domain_type::DomainType;
use super::intent::{IntentCatalog, IntentId};
use super::workflow::Workflow;

/// Conversation session - one user, one domain, one optional workflow.
///
/// State machine:
///   Idle -> Workflow (via workflow intent)
///   Workflow -> Idle (via cancel or completion)
///   Idle -> Idle (via informational/conversational intent)
///
/// You cannot start a workflow from Workflow state.
/// You cannot handle informational intents during a Workflow.
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: ConversationId,
    pub domain: DomainType,
    pub lang: String,
    pub state: ConversationState,
}

#[derive(Debug, Clone)]
pub enum ConversationState {
    Idle,
    Workflow(Workflow),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransitionResult {
    WorkflowStarted,
    WorkflowCancelled,
    NotAWorkflowIntent,
    IntentNotFound,
    /// A workflow is already active. Cancel it first.
    BlockedByActiveWorkflow,
}

impl Conversation {
    pub fn new(domain: DomainType) -> Self {
        Self {
            id: ConversationId::new(),
            domain,
            lang: "en".to_string(),
            state: ConversationState::Idle,
        }
    }

    pub fn with_id(id: ConversationId, domain: DomainType) -> Self {
        Self {
            id,
            domain,
            lang: "en".to_string(),
            state: ConversationState::Idle,
        }
    }

    /// Start a workflow intent. Only works from Idle.
    pub fn start_workflow(
        &mut self,
        intent: &IntentId,
        catalog: &IntentCatalog,
    ) -> TransitionResult {
        match catalog.get(intent) {
            None => TransitionResult::IntentNotFound,
            Some(_) if !catalog.is_workflow(intent) => TransitionResult::NotAWorkflowIntent,
            Some(_) => match &self.state {
                ConversationState::Idle => {
                    self.state = ConversationState::Workflow(Workflow::from_catalog(
                        intent.clone(),
                        catalog,
                    ));
                    TransitionResult::WorkflowStarted
                }
                ConversationState::Workflow(_) => TransitionResult::BlockedByActiveWorkflow,
            },
        }
    }

    /// Cancel the current workflow. Always returns to Idle.
    pub fn cancel_workflow(&mut self) -> TransitionResult {
        self.state = ConversationState::Idle;
        TransitionResult::WorkflowCancelled
    }

    pub fn complete_workflow(&mut self) {
        self.state = ConversationState::Idle;
    }

    pub fn active_workflow(&self) -> Option<&Workflow> {
        match &self.state {
            ConversationState::Workflow(wf) => Some(wf),
            ConversationState::Idle => None,
        }
    }

    pub fn active_workflow_mut(&mut self) -> Option<&mut Workflow> {
        match &mut self.state {
            ConversationState::Workflow(wf) => Some(wf),
            ConversationState::Idle => None,
        }
    }

    pub fn has_active_workflow(&self) -> bool {
        matches!(self.state, ConversationState::Workflow(_))
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.state, ConversationState::Idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::conversation::domain::intent::build_restaurant_catalog;

    fn catalog() -> IntentCatalog {
        build_restaurant_catalog()
    }

    #[test]
    fn starts_idle() {
        let conv = Conversation::new(DomainType::Restaurant);
        assert!(conv.is_idle());
        assert_eq!(conv.domain, DomainType::Restaurant);
    }

    #[test]
    fn start_book_workflow_from_idle() {
        let mut conv = Conversation::new(DomainType::Restaurant);
        let result = conv.start_workflow(&IntentId::new("reservation_create"), &catalog());
        assert_eq!(result, TransitionResult::WorkflowStarted);
        assert!(conv.has_active_workflow());
    }

    #[test]
    fn cannot_start_workflow_during_workflow() {
        let c = catalog();
        let mut conv = Conversation::new(DomainType::Restaurant);
        conv.start_workflow(&IntentId::new("reservation_create"), &c);

        let result = conv.start_workflow(&IntentId::new("reservation_cancel"), &c);
        assert_eq!(result, TransitionResult::BlockedByActiveWorkflow);
    }

    #[test]
    fn cancel_returns_to_idle() {
        let mut conv = Conversation::new(DomainType::Restaurant);
        conv.start_workflow(&IntentId::new("reservation_create"), &catalog());
        conv.cancel_workflow();
        assert!(conv.is_idle());
    }

    #[test]
    fn cancel_then_start_new_workflow() {
        let c = catalog();
        let mut conv = Conversation::new(DomainType::Restaurant);
        conv.start_workflow(&IntentId::new("reservation_create"), &c);
        conv.cancel_workflow();

        let result = conv.start_workflow(&IntentId::new("reservation_cancel"), &c);
        assert_eq!(result, TransitionResult::WorkflowStarted);
    }

    #[test]
    fn informational_intent_cannot_start_workflow() {
        let mut conv = Conversation::new(DomainType::Restaurant);
        let result = conv.start_workflow(&IntentId::new("ask_menu_general"), &catalog());
        assert_eq!(result, TransitionResult::NotAWorkflowIntent);
        assert!(conv.is_idle());
    }

    #[test]
    fn unknown_intent_returns_not_found() {
        let mut conv = Conversation::new(DomainType::Restaurant);
        let result = conv.start_workflow(&IntentId::new("fly_to_moon"), &catalog());
        assert_eq!(result, TransitionResult::IntentNotFound);
    }

    #[test]
    fn domain_is_fixed() {
        let conv = Conversation::new(DomainType::Hotel);
        assert_eq!(conv.domain, DomainType::Hotel);
    }
}
