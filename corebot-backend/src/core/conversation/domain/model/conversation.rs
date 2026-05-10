use crate::core::conversation::domain::model::conversation_id::ConversationId;
use crate::core::conversation::domain::model::domain_type::DomainType;
use crate::core::conversation::domain::model::intent::{IntentKind, IntentPolicy, NluTask};
use crate::core::conversation::domain::model::slot::{SlotError, SlotName, SlotValue};
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
pub enum StartWorkflowError {
    NotAWorkflowIntent,
    ActiveWorkflowAlreadyExists,
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

    pub fn with_started_workflow(
        &self,
        policy: &IntentPolicy,
    ) -> Result<Conversation, StartWorkflowError> {
        match policy.kind {
            IntentKind::Informational => Err(StartWorkflowError::NotAWorkflowIntent),
            IntentKind::Workflow => match &self.state {
                ConversationState::Idle => {
                    let mut updated_conversation = self.clone();
                    updated_conversation.state =
                        ConversationState::Workflow(Workflow::from_policy(policy));
                    Ok(updated_conversation)
                }
                ConversationState::Workflow(_) => {
                    Err(StartWorkflowError::ActiveWorkflowAlreadyExists)
                }
            },
        }
    }

    pub fn with_cancelled_workflow(&self) -> Conversation {
        let mut updated_conversation = self.clone();
        updated_conversation.state = ConversationState::Idle;
        updated_conversation
    }

    pub fn with_completed_workflow(&self) -> Conversation {
        let mut updated_conversation = self.clone();
        updated_conversation.state = ConversationState::Idle;
        updated_conversation
    }

    pub fn with_confirmed_workflow(&self) -> Result<Conversation, SlotError> {
        self.with_workflow_slot(SlotName::Confirmation, SlotValue::Boolean(true))
    }

    pub fn with_workflow_slot(
        &self,
        slot_name: SlotName,
        value: SlotValue,
    ) -> Result<Conversation, SlotError> {
        match &self.state {
            ConversationState::Idle => Ok(self.clone()),
            ConversationState::Workflow(workflow) => {
                let updated_workflow = workflow.with_slot(slot_name, value)?;
                let mut updated_conversation = self.clone();
                updated_conversation.state = ConversationState::Workflow(updated_workflow);
                Ok(updated_conversation)
            }
        }
    }

    pub fn active_workflow(&self) -> Option<&Workflow> {
        match &self.state {
            ConversationState::Workflow(wf) => Some(wf),
            ConversationState::Idle => None,
        }
    }

    pub fn has_active_workflow(&self) -> bool {
        matches!(self.state, ConversationState::Workflow(_))
    }

    pub fn detect_task(&self) -> Option<NluTask> {
        let workflow = self.active_workflow()?;
        if workflow.is_ready_for_confirmation() {
            return Some(NluTask::Choice);
        }

        workflow.nlu_task
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.state, ConversationState::Idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::conversation::domain::model::intent::{IntentId, NluTask, i18n_key};
    use crate::core::conversation::domain::slot::{
        EntityType, SlotDefinition, SlotName, SlotType, SlotValue,
    };

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
        let conv = Conversation::new(DomainType::Restaurant);
        let updated = conv
            .with_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        assert!(updated.has_active_workflow());
        assert!(conv.is_idle());
    }

    #[test]
    fn cannot_start_workflow_during_workflow() {
        let conv = Conversation::new(DomainType::Restaurant)
            .with_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        let result = conv.with_started_workflow(&workflow_policy(IntentId::ReservationCancel));

        assert_eq!(
            result.unwrap_err(),
            StartWorkflowError::ActiveWorkflowAlreadyExists
        );
    }

    #[test]
    fn cancel_returns_to_idle() {
        let conv = Conversation::new(DomainType::Restaurant)
            .with_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        let updated = conv.with_cancelled_workflow();

        assert!(updated.is_idle());
        assert!(conv.has_active_workflow());
    }

    #[test]
    fn cancel_then_start_new_workflow() {
        let conv = Conversation::new(DomainType::Restaurant)
            .with_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap()
            .with_cancelled_workflow();

        let updated = conv
            .with_started_workflow(&workflow_policy(IntentId::ReservationCancel))
            .unwrap();

        assert!(updated.has_active_workflow());
    }

    #[test]
    fn informational_intent_cannot_start_workflow() {
        let conv = Conversation::new(DomainType::Restaurant);
        let result = conv.with_started_workflow(&informational_policy(IntentId::AskMenuGeneral));

        assert_eq!(result.unwrap_err(), StartWorkflowError::NotAWorkflowIntent);
        assert!(conv.is_idle());
    }

    #[test]
    fn domain_is_fixed() {
        let conv = Conversation::new(DomainType::Hotel);
        assert_eq!(conv.domain, DomainType::Hotel);
    }

    #[test]
    fn workflow_slot_update_returns_updated_conversation() {
        let conv = Conversation::new(DomainType::Restaurant)
            .with_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        let updated = conv
            .with_workflow_slot(SlotName::Name, SlotValue::Text("Alice".to_string()))
            .unwrap();

        assert_eq!(
            updated
                .active_workflow()
                .and_then(|workflow| workflow.slot_value(SlotName::Name)),
            Some(&SlotValue::Text("Alice".to_string()))
        );
        assert_eq!(
            conv.active_workflow()
                .and_then(|workflow| workflow.slot_value(SlotName::Name)),
            None
        );
    }

    #[test]
    fn workflow_slot_update_reports_invalid_slot() {
        let conv = Conversation::new(DomainType::Restaurant)
            .with_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        let result = conv.with_workflow_slot(SlotName::Name, SlotValue::Text(String::new()));

        assert_eq!(result.unwrap_err().slot, SlotName::Name);
    }

    #[test]
    fn detect_task_returns_workflow_task_while_collecting_slots() {
        let conversation = Conversation::new(DomainType::Restaurant)
            .with_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        assert_eq!(conversation.detect_task(), Some(NluTask::ReservationCreate));
    }

    #[test]
    fn detect_task_returns_choice_when_ready_for_confirmation() {
        let conversation = Conversation::new(DomainType::Restaurant)
            .with_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap()
            .with_workflow_slot(SlotName::Name, SlotValue::Text("Alice".to_string()))
            .unwrap();

        assert_eq!(conversation.detect_task(), Some(NluTask::Choice));
    }
}
