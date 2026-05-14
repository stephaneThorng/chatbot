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
    known_customer_name: Option<String>,
    last_reservation_reference: Option<String>,
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

#[derive(Debug, PartialEq)]
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
            known_customer_name: None,
            last_reservation_reference: None,
            state: ConversationState::Idle,
        }
    }

    pub fn with_id(id: ConversationId, domain: DomainType) -> Self {
        Self {
            id,
            domain,
            lang: "en".to_string(),
            known_customer_name: None,
            last_reservation_reference: None,
            state: ConversationState::Idle,
        }
    }

    pub fn into_started_workflow(
        mut self,
        policy: &IntentPolicy,
    ) -> Result<Conversation, StartWorkflowError> {
        match policy.kind {
            IntentKind::Informational => Err(StartWorkflowError::NotAWorkflowIntent),
            IntentKind::Workflow if self.has_active_workflow() => {
                Err(StartWorkflowError::ActiveWorkflowAlreadyExists)
            }
            IntentKind::Workflow => {
                self.state = ConversationState::Workflow(Workflow::from_policy(policy));
                Ok(self)
            }
        }
    }

    pub fn into_cancelled_workflow(mut self) -> Conversation {
        self.state = ConversationState::Idle;
        self
    }

    pub fn into_completed_workflow(mut self) -> Conversation {
        self.state = ConversationState::Idle;
        self
    }

    pub fn into_confirmed_workflow(self) -> Result<Conversation, SlotError> {
        self.into_workflow_slot(SlotName::Confirmation, SlotValue::Boolean(true))
    }

    pub fn into_workflow_slot(
        mut self,
        slot_name: SlotName,
        value: SlotValue,
    ) -> Result<Conversation, SlotError> {
        self.set_workflow_slot(slot_name, value)?;
        Ok(self)
    }

    pub fn set_workflow_slot(
        &mut self,
        slot_name: SlotName,
        value: SlotValue,
    ) -> Result<(), SlotError> {
        match &mut self.state {
            ConversationState::Idle => Ok(()),
            ConversationState::Workflow(workflow) => workflow.set_slot(slot_name, value),
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

    pub fn known_customer_name(&self) -> Option<&str> {
        self.known_customer_name.as_deref()
    }

    pub fn last_reservation_reference(&self) -> Option<&str> {
        self.last_reservation_reference.as_deref()
    }

    pub fn remember_customer_name(&mut self, name: impl Into<String>) {
        self.known_customer_name = Some(name.into());
    }

    pub fn remember_reservation_reference(&mut self, reference: impl Into<String>) {
        self.last_reservation_reference = Some(reference.into());
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
            .clone()
            .into_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        assert!(updated.has_active_workflow());
        assert!(conv.is_idle());
    }

    #[test]
    fn cannot_start_workflow_during_workflow() {
        let conv = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        let result = conv.into_started_workflow(&workflow_policy(IntentId::ReservationCancel));

        assert_eq!(
            result.unwrap_err(),
            StartWorkflowError::ActiveWorkflowAlreadyExists
        );
    }

    #[test]
    fn cancel_returns_to_idle() {
        let conv = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        let updated = conv.clone().into_cancelled_workflow();

        assert!(updated.is_idle());
        assert!(conv.has_active_workflow());
    }

    #[test]
    fn cancel_then_start_new_workflow() {
        let conv = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap()
            .into_cancelled_workflow();

        let updated = conv
            .into_started_workflow(&workflow_policy(IntentId::ReservationCancel))
            .unwrap();

        assert!(updated.has_active_workflow());
    }

    #[test]
    fn informational_intent_cannot_start_workflow() {
        let conv = Conversation::new(DomainType::Restaurant);
        let result = conv
            .clone()
            .into_started_workflow(&informational_policy(IntentId::AskMenuGeneral));

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
            .into_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        let updated = conv
            .clone()
            .into_workflow_slot(SlotName::Name, SlotValue::Text("Alice".to_string()))
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
            .into_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        let result = conv.into_workflow_slot(SlotName::Name, SlotValue::Text(String::new()));

        assert_eq!(result.unwrap_err().slot, SlotName::Name);
    }

    #[test]
    fn detect_task_returns_workflow_task_while_collecting_slots() {
        let conversation = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap();

        assert_eq!(conversation.detect_task(), Some(NluTask::ReservationCreate));
    }

    #[test]
    fn detect_task_returns_choice_when_ready_for_confirmation() {
        let conversation = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_policy(IntentId::ReservationCreate))
            .unwrap()
            .into_workflow_slot(SlotName::Name, SlotValue::Text("Alice".to_string()))
            .unwrap();

        assert_eq!(conversation.detect_task(), Some(NluTask::Choice));
    }
}
