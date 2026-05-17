use crate::core::conversation::domain::model::conversation_id::ConversationId;
use crate::core::conversation::domain::model::domain_type::DomainType;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentWorkflow, NluTask};
use crate::core::conversation::domain::model::slot::{SlotDataValue, SlotError, SlotName};
use crate::core::conversation::domain::model::workflow::Workflow;
use crate::core::conversation::domain::workflow::NextSlot;
use uuid::Uuid;

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
    pub business_id: Uuid,
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
        Self::new_for_business(domain, Uuid::nil())
    }

    pub fn new_for_business(domain: DomainType, business_id: Uuid) -> Self {
        Self {
            id: ConversationId::new(),
            domain,
            business_id,
            lang: "en".to_string(),
            known_customer_name: None,
            last_reservation_reference: None,
            state: ConversationState::Idle,
        }
    }

    pub fn with_id(id: ConversationId, domain: DomainType) -> Self {
        Self::with_id_for_business(id, domain, Uuid::nil())
    }

    pub fn with_id_for_business(id: ConversationId, domain: DomainType, business_id: Uuid) -> Self {
        Self {
            id,
            domain,
            business_id,
            lang: "en".to_string(),
            known_customer_name: None,
            last_reservation_reference: None,
            state: ConversationState::Idle,
        }
    }

    pub fn into_started_workflow(
        mut self,
        config: &IntentConfig,
    ) -> Result<Conversation, StartWorkflowError> {
        match &config.workflow {
            IntentWorkflow::Informational => Err(StartWorkflowError::NotAWorkflowIntent),
            IntentWorkflow::Workflow(_) if self.has_active_workflow() => {
                Err(StartWorkflowError::ActiveWorkflowAlreadyExists)
            }
            IntentWorkflow::Workflow(_) => {
                self.state = ConversationState::Workflow(Workflow::from_config(config));
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

    pub fn into_confirmed_workflow(mut self) -> Conversation {
        if let ConversationState::Workflow(ref mut wf) = self.state {
            wf.confirm();
        }
        self
    }

    pub fn into_reopened_workflow(mut self) -> Conversation {
        if let ConversationState::Workflow(ref mut wf) = self.state {
            wf.reopen_confirmation();
        }
        self
    }

    pub fn into_workflow_slot(
        mut self,
        slot_name: SlotName,
        value: SlotDataValue,
    ) -> Result<Conversation, SlotError> {
        self.set_workflow_slot(slot_name, value)?;
        Ok(self)
    }

    pub fn set_workflow_slot(
        &mut self,
        slot_name: SlotName,
        value: SlotDataValue,
    ) -> Result<(), SlotError> {
        match &mut self.state {
            ConversationState::Idle => Ok(()),
            ConversationState::Workflow(workflow) => workflow.set_slot(slot_name, value),
        }
    }

    /// Remove a slot value from the active workflow (e.g. after a constraint violation).
    pub fn clear_workflow_slot(&mut self, slot_name: SlotName) {
        if let ConversationState::Workflow(workflow) = &mut self.state {
            workflow.clear_slot(slot_name);
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

    pub fn detect_slot_hint(&self) -> Option<SlotName> {
        let workflow = self.active_workflow()?;
        match workflow.next_required_slot()? {
            NextSlot::Data(slot) => Some(slot.name),
            NextSlot::Confirmation => None,
        }
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
    use crate::core::conversation::domain::model::intent::{
        IntentConfig, IntentId, IntentWorkflow, NluTask, WorkflowConfig, i18n_key,
    };
    use crate::core::conversation::domain::model::slot::{SlotConfig, SlotDataValue, SlotName};

    fn workflow_config(intent: IntentId) -> IntentConfig {
        IntentConfig {
            id: intent,
            workflow: IntentWorkflow::Workflow(WorkflowConfig {
                nlu_task: Some(NluTask::ReservationCreate),
                slots: vec![SlotConfig {
                    name: SlotName::Name,
                    required: true,
                    prompt: i18n_key("test.prompt"),
                    constraints: vec![],
                }],
                starting_message: None,
                confirmation_prompt: None,
                completion_response: None,
            }),
        }
    }

    fn informational_config(intent: IntentId) -> IntentConfig {
        IntentConfig {
            id: intent,
            workflow: IntentWorkflow::Informational,
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
            .into_started_workflow(&workflow_config(IntentId::ReservationCreate))
            .unwrap();
        assert!(updated.has_active_workflow());
        assert!(conv.is_idle());
    }

    #[test]
    fn cannot_start_workflow_during_workflow() {
        let conv = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_config(IntentId::ReservationCreate))
            .unwrap();
        let result = conv.into_started_workflow(&workflow_config(IntentId::ReservationCancel));
        assert_eq!(
            result.unwrap_err(),
            StartWorkflowError::ActiveWorkflowAlreadyExists
        );
    }

    #[test]
    fn cancel_returns_to_idle() {
        let conv = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_config(IntentId::ReservationCreate))
            .unwrap();
        let updated = conv.clone().into_cancelled_workflow();
        assert!(updated.is_idle());
        assert!(conv.has_active_workflow());
    }

    #[test]
    fn cancel_then_start_new_workflow() {
        let conv = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_config(IntentId::ReservationCreate))
            .unwrap()
            .into_cancelled_workflow();
        let updated = conv
            .into_started_workflow(&workflow_config(IntentId::ReservationCancel))
            .unwrap();
        assert!(updated.has_active_workflow());
    }

    #[test]
    fn informational_intent_cannot_start_workflow() {
        let conv = Conversation::new(DomainType::Restaurant);
        let result = conv
            .clone()
            .into_started_workflow(&informational_config(IntentId::AskMenuGeneral));
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
            .into_started_workflow(&workflow_config(IntentId::ReservationCreate))
            .unwrap();
        let updated = conv
            .clone()
            .into_workflow_slot(SlotName::Name, SlotDataValue::Text("Alice".to_string()))
            .unwrap();
        assert_eq!(
            updated
                .active_workflow()
                .and_then(|wf| wf.slot_value(SlotName::Name)),
            Some(&SlotDataValue::Text("Alice".to_string()))
        );
        assert_eq!(
            conv.active_workflow()
                .and_then(|wf| wf.slot_value(SlotName::Name)),
            None
        );
    }

    #[test]
    fn workflow_slot_update_reports_invalid_slot() {
        let conv = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_config(IntentId::ReservationCreate))
            .unwrap();
        // Providing a Number where Text is expected triggers a type mismatch error.
        let result = conv.into_workflow_slot(SlotName::Name, SlotDataValue::Number(42));
        assert_eq!(result.unwrap_err().slot, SlotName::Name);
    }

    #[test]
    fn detect_task_returns_workflow_task_while_collecting_slots() {
        let conversation = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_config(IntentId::ReservationCreate))
            .unwrap();
        assert_eq!(conversation.detect_task(), Some(NluTask::ReservationCreate));
        assert_eq!(conversation.detect_slot_hint(), Some(SlotName::Name));
    }

    #[test]
    fn detect_task_returns_choice_when_ready_for_confirmation() {
        let conversation = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_config(IntentId::ReservationCreate))
            .unwrap()
            .into_workflow_slot(SlotName::Name, SlotDataValue::Text("Alice".to_string()))
            .unwrap();

        assert_eq!(conversation.detect_task(), Some(NluTask::Choice));
        assert_eq!(conversation.detect_slot_hint(), None);
    }
}
