use std::collections::HashMap;

use rust_i18n::t;

use crate::core::conversation::domain::conversation::Conversation;
use crate::core::conversation::domain::date_resolver::{DateResolveError, resolve_date};
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::slot::{
    EntityType, SlotConstraint, SlotDefinition, SlotName, SlotType, SlotValue,
};
use crate::core::conversation::domain::workflow::NextSlot;
use crate::core::nlu_engine::domain::analysis::NluEntity;

/// Stateless input passed to an intent-specific handler.
///
/// Handlers can inspect the current user text and NER entities, but they do not
/// receive mutable conversation state. Workflow-capable handlers return an
/// updated conversation instead of mutating the original one.
pub struct IntentHandlerInput<'a> {
    pub conversation: Conversation,
    pub analysis_intent: &'a IntentId,
    pub text: &'a str,
    pub analysis_entities: &'a [NluEntity],
}

/// Immediate result produced by an informational intent handler.
///
/// This type is intentionally small for v1, but keeps handler output typed so
/// later metadata can be added without changing the processor contract.
pub struct StateHandlerResult {
    pub updated_conversation: Conversation,
    pub reply: String,
    pub handled_intent: IntentId,
}

pub enum WorkflowPostProcessResult {
    Succeeded {
        updated_conversation: Conversation,
        reply: Option<String>,
    },
    Failed {
        updated_conversation: Conversation,
        reply: String,
    },
}

/// Stateless application component that handles one immediate intent.
pub trait IntentHandler: Send + Sync {
    fn intent(&self) -> IntentId;
    fn policy(&self) -> IntentPolicy;
    fn is_workflow(&self) -> bool {
        self.policy().kind == IntentKind::Workflow
    }
    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult;

    fn lookup_entity_value<'a>(
        &self,
        input: &'a IntentHandlerInput<'a>,
        entity_type: EntityType,
    ) -> Option<&'a str> {
        input
            .analysis_entities
            .iter()
            .find(|entity| entity.entity_type == entity_type)
            .map(|entity| entity.value.as_str())
    }

    fn handle_workflow(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let mut updated_conversation = input.conversation;
        let policy = self.policy();

        let just_started = !updated_conversation.has_active_workflow();
        if just_started {
            updated_conversation = updated_conversation
                .into_started_workflow(&policy)
                .expect("workflow handler policy must start a workflow");
        }

        if input.analysis_intent == &IntentId::Cancel {
            updated_conversation = updated_conversation.into_cancelled_workflow();
            let reply = t!(
                "system.workflow_cancelled",
                locale = updated_conversation.lang.as_str()
            )
            .to_string();
            return StateHandlerResult {
                updated_conversation,
                reply,
                handled_intent: self.intent(),
            };
        }

        let extracted_updates =
            self.extract_slot_updates(&updated_conversation, input.text, input.analysis_entities);
        let update_result = self.apply_slot_updates(updated_conversation, extracted_updates);
        let updated_any_slot = update_result.updated_any_slot;
        let invalid_slot = update_result.invalid_slot;
        updated_conversation = update_result.updated_conversation;

        if let Some(invalid_slot) = invalid_slot {
            let reply = self.slot_prompt(&policy, invalid_slot, updated_conversation.lang.as_str());
            return StateHandlerResult {
                updated_conversation,
                reply,
                handled_intent: self.intent(),
            };
        }

        // Evaluate declarative slot constraints; re-prompt on first violation.
        if let Some(violation) = self.validate_constraints(&policy, &mut updated_conversation) {
            return StateHandlerResult {
                updated_conversation,
                reply: violation,
                handled_intent: self.intent(),
            };
        }

        let ready_for_confirmation = updated_conversation
            .active_workflow()
            .is_some_and(|workflow| workflow.is_ready_for_confirmation());

        if !ready_for_confirmation {
            let slot_prompt = self.next_slot_prompt(&updated_conversation, &policy);
            let reply = if just_started {
                self.starting_reply(&policy, updated_conversation.lang.as_str(), &slot_prompt)
            } else {
                slot_prompt
            };
            return StateHandlerResult {
                updated_conversation,
                reply,
                handled_intent: self.intent(),
            };
        }

        match input.analysis_intent {
            IntentId::Affirmative if !updated_any_slot => {
                let lang = updated_conversation.lang.clone();
                match self.post_process(lang.as_str(), updated_conversation) {
                    WorkflowPostProcessResult::Succeeded {
                        mut updated_conversation,
                        reply,
                    } => {
                        let lang = updated_conversation.lang.clone();
                        updated_conversation = updated_conversation.into_completed_workflow();
                        let reply =
                            reply.unwrap_or_else(|| self.completion_reply(&policy, lang.as_str()));
                        StateHandlerResult {
                            updated_conversation,
                            reply,
                            handled_intent: self.intent(),
                        }
                    }
                    WorkflowPostProcessResult::Failed {
                        updated_conversation,
                        reply,
                    } => StateHandlerResult {
                        updated_conversation,
                        reply,
                        handled_intent: self.intent(),
                    },
                }
            }
            IntentId::Negative if !updated_any_slot => {
                let reply = self.negative_prompt(updated_conversation.lang.as_str());
                StateHandlerResult {
                    updated_conversation,
                    reply,
                    handled_intent: self.intent(),
                }
            }
            _ => {
                let reply = self.confirmation_prompt(&policy, &updated_conversation);
                StateHandlerResult {
                    updated_conversation,
                    reply,
                    handled_intent: self.intent(),
                }
            }
        }
    }

    fn post_process(&self, _lang: &str, conversation: Conversation) -> WorkflowPostProcessResult {
        WorkflowPostProcessResult::Succeeded {
            updated_conversation: conversation,
            reply: None,
        }
    }

    fn negative_prompt(&self, lang: &str) -> String {
        t!("system.workflow_update_prompt", locale = lang).to_string()
    }

    fn next_slot_prompt(&self, conversation: &Conversation, policy: &IntentPolicy) -> String {
        let Some(workflow) = conversation.active_workflow() else {
            return self.confirmation_prompt(policy, conversation);
        };
        match workflow.next_required_slot() {
            Some(NextSlot::Data(definition)) => {
                self.slot_prompt(policy, definition.name, conversation.lang.as_str())
            }
            Some(NextSlot::Confirmation) | None => self.confirmation_prompt(policy, conversation),
        }
    }

    fn slot_prompt(&self, policy: &IntentPolicy, slot_name: SlotName, lang: &str) -> String {
        policy
            .workflow_slots
            .iter()
            .find(|slot| slot.name == slot_name)
            .map(|slot| t!(slot.prompt.0.as_str(), locale = lang).to_string())
            .unwrap_or_else(|| {
                t!(
                    "system.missing_slot_fallback",
                    locale = lang,
                    slot = slot_name.as_str()
                )
                .to_string()
            })
    }

    fn confirmation_prompt(&self, policy: &IntentPolicy, conversation: &Conversation) -> String {
        policy
            .confirmation_prompt
            .as_ref()
            .map(|key| t!(key.0.as_str(), locale = conversation.lang.as_str()).to_string())
            .unwrap_or_else(|| {
                t!(
                    "system.confirm_generic",
                    locale = conversation.lang.as_str()
                )
                .to_string()
            })
    }

    fn completion_reply(&self, policy: &IntentPolicy, lang: &str) -> String {
        policy
            .completion_response
            .as_ref()
            .map(|key| t!(key.0.as_str(), locale = lang).to_string())
            .unwrap_or_else(|| t!("system.workflow_complete", locale = lang).to_string())
    }

    fn starting_reply(&self, policy: &IntentPolicy, lang: &str, slot_prompt: &str) -> String {
        match policy.starting_message.as_ref() {
            Some(key) => {
                let starting = t!(key.0.as_str(), locale = lang).to_string();
                format!("{}\n{}", starting, slot_prompt)
            }
            None => slot_prompt.to_string(),
        }
    }

    /// Evaluate all declarative constraints on currently filled slots.
    ///
    /// Returns the translated error message for the first violation found, and
    /// clears the offending slot from the conversation so the user is re-prompted.
    /// Returns `None` when all constraints pass.
    fn validate_constraints(
        &self,
        policy: &IntentPolicy,
        conversation: &mut Conversation,
    ) -> Option<String> {
        let lang = conversation.lang.clone();
        for slot_def in &policy.workflow_slots {
            let Some(value) = conversation
                .active_workflow()
                .and_then(|wf| wf.slot_value(slot_def.name))
                .cloned()
            else {
                continue;
            };

            for entry in &slot_def.constraints {
                if let Some(error_key) = self.check_slot_constraint(&entry.constraint, &value) {
                    let resolved_key = entry
                        .error_key
                        .as_ref()
                        .map(|k| k.0.as_str())
                        .unwrap_or(error_key);
                    let reply = t!(resolved_key, locale = lang.as_str()).to_string();
                    conversation.clear_workflow_slot(slot_def.name);
                    return Some(reply);
                }
            }
        }
        None
    }

    /// Returns the fallback i18n error key if the constraint is violated, `None` if it passes.
    fn check_slot_constraint(
        &self,
        constraint: &SlotConstraint,
        value: &SlotValue,
    ) -> Option<&'static str> {
        match (constraint, value) {
            (SlotConstraint::TextMaxLen(max), SlotValue::Text(s)) => {
                if s.len() > *max {
                    Some(SlotConstraint::TextMaxLen(0).default_error_key())
                } else {
                    None
                }
            }
            (SlotConstraint::EmailFormat, SlotValue::Text(s)) => {
                let valid = s.contains('@')
                    && s.split('@').nth(0).is_some_and(|local| !local.is_empty())
                    && s.split('@').nth(1).is_some_and(|domain| domain.contains('.') && domain.len() > 2);
                if valid { None } else { Some(SlotConstraint::EmailFormat.default_error_key()) }
            }
            (SlotConstraint::NumberRange(min, max), SlotValue::Number(n)) => {
                if n < min || n > max {
                    Some(SlotConstraint::NumberRange(0, 0).default_error_key())
                } else {
                    None
                }
            }
            (SlotConstraint::FutureDate, SlotValue::Date(raw)) => {
                match resolve_date(raw) {
                    Ok(_) => None,
                    Err(DateResolveError::PastDate(_)) | Err(DateResolveError::Unparseable) => {
                        Some(SlotConstraint::FutureDate.default_error_key())
                    }
                }
            }
            _ => None,
        }
    }

    fn extract_slot_updates(
        &self,
        conversation: &Conversation,
        raw_text: &str,
        entities: &[NluEntity],
    ) -> SlotUpdateResult {
        let Some(workflow) = conversation.active_workflow() else {
            return SlotUpdateResult { updates: vec![] };
        };

        let mut updates = vec![];

        for entity in entities {
            for slot in workflow.slot_definitions() {
                if !slot
                    .entity_types
                    .iter()
                    .any(|entity_type| entity_type == &entity.entity_type)
                {
                    continue;
                }

                let Some(slot_value) = self.slot_value_from_entity(slot, entity.value.as_str())
                else {
                    continue;
                };

                updates.push(SlotUpdate {
                    slot_name: slot.name,
                    value: slot_value,
                });
            }
        }

        if updates.is_empty()
            && let Some(NextSlot::Data(next_slot)) = workflow.next_required_slot()
            && next_slot.slot_type == SlotType::Number
            && let Some(number) = self.parse_number_slot(raw_text)
        {
            updates.push(SlotUpdate {
                slot_name: next_slot.name,
                value: SlotValue::Number(number),
            });
        }

        SlotUpdateResult { updates }
    }

    fn apply_slot_updates(
        &self,
        conversation: Conversation,
        extracted_updates: SlotUpdateResult,
    ) -> SlotUpdateApplicationResult {
        let mut updated_conversation = conversation;
        let mut updated_any_slot = false;
        let mut invalid_slot = None;

        for update in extracted_updates.updates {
            if let Err(error) =
                updated_conversation.set_workflow_slot(update.slot_name, update.value)
            {
                invalid_slot = Some(error.slot);
                break;
            }
            updated_any_slot = true;
        }

        SlotUpdateApplicationResult {
            updated_conversation,
            updated_any_slot,
            invalid_slot,
        }
    }

    fn slot_value_from_entity(&self, slot: &SlotDefinition, raw_value: &str) -> Option<SlotValue> {
        match slot.slot_type {
            SlotType::Text => Some(SlotValue::Text(raw_value.to_string())),
            SlotType::Date => Some(SlotValue::Date(raw_value.to_string())),
            SlotType::Time => Some(SlotValue::Time(raw_value.to_string())),
            SlotType::Number => self.parse_number_slot(raw_value).map(SlotValue::Number),
            SlotType::Boolean => None,
        }
    }

    fn parse_number_slot(&self, raw_value: &str) -> Option<u32> {
        let digits = raw_value
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>();
        digits.parse().ok()
    }
}

pub struct SlotUpdateResult {
    pub updates: Vec<SlotUpdate>,
}

pub struct SlotUpdateApplicationResult {
    pub updated_conversation: Conversation,
    pub updated_any_slot: bool,
    pub invalid_slot: Option<SlotName>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlotUpdate {
    pub slot_name: SlotName,
    pub value: SlotValue,
}

/// Lookup table for application-level intent handlers.
pub struct IntentHandlerRegistry<'a> {
    handlers: HashMap<IntentId, Box<dyn IntentHandler + 'a>>,
}

impl<'a> IntentHandlerRegistry<'a> {
    pub fn new(handlers: Vec<Box<dyn IntentHandler + 'a>>) -> Self {
        Self {
            handlers: handlers
                .into_iter()
                .map(|handler| (handler.intent(), handler))
                .collect(),
        }
    }

    pub fn get(&self, intent: &IntentId) -> Option<&(dyn IntentHandler + 'a)> {
        self.handlers.get(intent).map(Box::as_ref)
    }

    pub fn find_policy(&self, intent: &IntentId) -> Option<IntentPolicy> {
        self.get(intent).map(IntentHandler::policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::conversation::domain::domain_type::DomainType;

    struct StubHandler;

    impl IntentHandler for StubHandler {
        fn intent(&self) -> IntentId {
            IntentId::AskOpeningHours
        }

        fn policy(&self) -> IntentPolicy {
            IntentPolicy {
                id: self.intent(),
                kind: crate::core::conversation::domain::intent::IntentKind::Informational,
                nlu_task: None,
                workflow_slots: vec![],
                starting_message: None,
                confirmation_prompt: None,
                completion_response: None,
            }
        }

        fn handle(&self, _input: IntentHandlerInput<'_>) -> StateHandlerResult {
            StateHandlerResult {
                updated_conversation: Conversation::new(DomainType::Restaurant),
                reply: "handled".to_string(),
                handled_intent: self.intent(),
            }
        }
    }

    #[test]
    fn registry_resolves_known_intent_handler() {
        let registry = IntentHandlerRegistry::new(vec![Box::new(StubHandler)]);

        assert!(registry.get(&IntentId::AskOpeningHours).is_some());
        assert!(registry.get(&IntentId::Greeting).is_none());
    }

    #[test]
    fn registry_resolves_known_intent_policy() {
        let registry = IntentHandlerRegistry::new(vec![Box::new(StubHandler)]);

        let policy = registry.find_policy(&IntentId::AskOpeningHours).unwrap();

        assert_eq!(policy.id, IntentId::AskOpeningHours);
    }
}
