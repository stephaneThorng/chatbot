use crate::core::conversation::domain::catalog::intent::{IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::conversation_id::ConversationId;
use crate::core::conversation::domain::model::domain_type::DomainType;
use crate::core::conversation::domain::model::workflow::Workflow;

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

/// Coarse state of a conversation session.
///
/// Detailed progress inside `Workflow` is modeled with generic slot
/// requirements instead of many fine-grained conversation states.
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
    pub fn start_workflow(&mut self, policy: &IntentPolicy) -> TransitionResult {
        match policy.kind {
            IntentKind::Informational => TransitionResult::NotAWorkflowIntent,
            IntentKind::Workflow => match &self.state {
                ConversationState::Idle => {
                    self.state = ConversationState::Workflow(Workflow::from_policy(policy));
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
    use crate::core::conversation::domain::intent::{IntentId, NluTask, i18n_key};
    use crate::core::conversation::domain::slot::{EntityType, SlotDefinition, SlotName, SlotType};

    fn workflow_policy(intent: IntentId) -> IntentPolicy {
        IntentPolicy {
            id: intent,
            kind: IntentKind::Workflow,
            nlu_task: Some(NluTask::ReservationCreate),
            workflow_slots: vec![SlotDefinition {
                name: SlotName::Name,
                slot_type: SlotType::Text,
                required: true,
                entity_types: vec![EntityType::Person],
                prompt: i18n_key("test.prompt"),
            }],
            supported_entities: vec![EntityType::Person],
            confirmation_prompt: None,
            completion_response: None,
        }
    }

    fn informational_policy(intent: IntentId) -> IntentPolicy {
        IntentPolicy {
            id: intent,
            kind: IntentKind::Informational,
            nlu_task: None,
            workflow_slots: vec![],
            supported_entities: vec![],
            confirmation_prompt: None,
            completion_response: None,
        }
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
        let result = conv.start_workflow(&workflow_policy(IntentId::ReservationCreate));
        assert_eq!(result, TransitionResult::WorkflowStarted);
        assert!(conv.has_active_workflow());
    }

    #[test]
    fn cannot_start_workflow_during_workflow() {
        let mut conv = Conversation::new(DomainType::Restaurant);
        conv.start_workflow(&workflow_policy(IntentId::ReservationCreate));

        let result = conv.start_workflow(&workflow_policy(IntentId::ReservationCancel));
        assert_eq!(result, TransitionResult::BlockedByActiveWorkflow);
    }

    #[test]
    fn cancel_returns_to_idle() {
        let mut conv = Conversation::new(DomainType::Restaurant);
        conv.start_workflow(&workflow_policy(IntentId::ReservationCreate));
        conv.cancel_workflow();
        assert!(conv.is_idle());
    }

    #[test]
    fn cancel_then_start_new_workflow() {
        let mut conv = Conversation::new(DomainType::Restaurant);
        conv.start_workflow(&workflow_policy(IntentId::ReservationCreate));
        conv.cancel_workflow();

        let result = conv.start_workflow(&workflow_policy(IntentId::ReservationCancel));
        assert_eq!(result, TransitionResult::WorkflowStarted);
    }

    #[test]
    fn informational_intent_cannot_start_workflow() {
        let mut conv = Conversation::new(DomainType::Restaurant);
        let result = conv.start_workflow(&informational_policy(IntentId::AskMenuGeneral));
        assert_eq!(result, TransitionResult::NotAWorkflowIntent);
        assert!(conv.is_idle());
    }

    #[test]
    fn domain_is_fixed() {
        let conv = Conversation::new(DomainType::Hotel);
        assert_eq!(conv.domain, DomainType::Hotel);
    }
}
