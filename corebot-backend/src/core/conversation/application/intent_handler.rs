use std::collections::HashMap;
use std::sync::Arc;

use rust_i18n::t;

use crate::core::conversation::domain::conversation::Conversation;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::slot::{
    EntityType, SlotDefinition, SlotName, SlotType, SlotValue,
};
use crate::core::conversation::domain::workflow::NextSlot;
use crate::core::nlu_engine::domain::analysis::NluEntity;

/// Stateless input passed to an intent-specific handler.
///
/// Handlers can inspect the current user text and NER entities, but they do not
/// receive mutable conversation state. Workflow-capable handlers return an
/// updated conversation instead of mutating the original one.
pub struct IntentHandlerInput<'a> {
    pub conversation: &'a Conversation,
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
    Succeeded { reply: Option<String> },
    Failed { reply: String },
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
        let mut updated_conversation = input.conversation.clone();

        if !updated_conversation.has_active_workflow()
            && let Ok(started_conversation) =
                updated_conversation.with_started_workflow(&self.policy())
        {
            updated_conversation = started_conversation;
        }

        if input.analysis_intent == &IntentId::Cancel {
            updated_conversation = updated_conversation.with_cancelled_workflow();
            return StateHandlerResult {
                updated_conversation,
                reply: t!(
                    "system.workflow_cancelled",
                    locale = input.conversation.lang.as_str()
                )
                .to_string(),
                handled_intent: self.intent(),
            };
        }

        let extracted_updates =
            self.extract_slot_updates(&updated_conversation, input.analysis_entities);
        let update_result = self.apply_slot_updates(&updated_conversation, extracted_updates);
        updated_conversation = update_result.updated_conversation.clone();

        if let Some(invalid_slot) = update_result.invalid_slot {
            return StateHandlerResult {
                updated_conversation,
                reply: self.slot_prompt(invalid_slot, input.conversation.lang.as_str()),
                handled_intent: self.intent(),
            };
        }

        let ready_for_confirmation = updated_conversation
            .active_workflow()
            .is_some_and(|workflow| workflow.is_ready_for_confirmation());

        if !ready_for_confirmation {
            let reply =
                self.next_slot_prompt(&updated_conversation, input.conversation.lang.as_str());
            return StateHandlerResult {
                updated_conversation,
                reply,
                handled_intent: self.intent(),
            };
        }

        match input.analysis_intent {
            IntentId::Affirmative if !update_result.updated_any_slot => {
                match self.post_process(input.conversation.lang.as_str(), &updated_conversation) {
                    WorkflowPostProcessResult::Succeeded { reply } => {
                        updated_conversation = updated_conversation.with_completed_workflow();
                        StateHandlerResult {
                            updated_conversation,
                            reply: reply.unwrap_or_else(|| {
                                self.completion_reply(input.conversation.lang.as_str())
                            }),
                            handled_intent: self.intent(),
                        }
                    }
                    WorkflowPostProcessResult::Failed { reply } => StateHandlerResult {
                        updated_conversation,
                        reply,
                        handled_intent: self.intent(),
                    },
                }
            }
            IntentId::Negative if !update_result.updated_any_slot => StateHandlerResult {
                updated_conversation,
                reply: self.negative_prompt(input.conversation.lang.as_str()),
                handled_intent: self.intent(),
            },
            _ => StateHandlerResult {
                updated_conversation,
                reply: self.confirmation_prompt(input.conversation.lang.as_str()),
                handled_intent: self.intent(),
            },
        }
    }

    fn post_process(&self, _lang: &str, _conversation: &Conversation) -> WorkflowPostProcessResult {
        WorkflowPostProcessResult::Succeeded { reply: None }
    }

    fn negative_prompt(&self, lang: &str) -> String {
        t!("system.workflow_update_prompt", locale = lang).to_string()
    }

    fn next_slot_prompt(&self, conversation: &Conversation, lang: &str) -> String {
        let Some(workflow) = conversation.active_workflow() else {
            return self.confirmation_prompt(lang);
        };
        match workflow.next_required_slot() {
            Some(NextSlot::Data(definition)) => self.slot_prompt(definition.name, lang),
            Some(NextSlot::Confirmation) | None => self.confirmation_prompt(lang),
        }
    }

    fn slot_prompt(&self, slot_name: SlotName, lang: &str) -> String {
        self.policy()
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

    fn confirmation_prompt(&self, lang: &str) -> String {
        self.policy()
            .confirmation_prompt
            .map(|key| t!(key.0.as_str(), locale = lang).to_string())
            .unwrap_or_else(|| t!("system.confirm_generic", locale = lang).to_string())
    }

    fn completion_reply(&self, lang: &str) -> String {
        self.policy()
            .completion_response
            .map(|key| t!(key.0.as_str(), locale = lang).to_string())
            .unwrap_or_else(|| t!("system.workflow_complete", locale = lang).to_string())
    }

    fn extract_slot_updates(
        &self,
        conversation: &Conversation,
        entities: &[NluEntity],
    ) -> SlotUpdateResult {
        let Some(workflow) = conversation.active_workflow() else {
            return SlotUpdateResult { updates: vec![] };
        };

        let slot_definitions = workflow.slot_definitions().to_vec();
        let mut updates = vec![];

        for entity in entities {
            for slot in &slot_definitions {
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

        SlotUpdateResult { updates }
    }

    fn apply_slot_updates(
        &self,
        conversation: &Conversation,
        extracted_updates: SlotUpdateResult,
    ) -> SlotUpdateApplicationResult {
        let mut updated_conversation = conversation.clone();
        let mut updated_any_slot = false;
        let mut invalid_slot = None;

        for update in extracted_updates.updates {
            match updated_conversation.with_workflow_slot(update.slot_name, update.value) {
                Ok(conversation) => {
                    updated_conversation = conversation;
                    updated_any_slot = true;
                }
                Err(error) => {
                    invalid_slot = Some(error.slot);
                    break;
                }
            }
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
pub struct IntentHandlerRegistry {
    handlers: HashMap<IntentId, Arc<dyn IntentHandler>>,
}

impl IntentHandlerRegistry {
    pub fn new(handlers: Vec<Arc<dyn IntentHandler>>) -> Self {
        Self {
            handlers: handlers
                .into_iter()
                .map(|handler| (handler.intent(), handler))
                .collect(),
        }
    }

    pub fn get(&self, intent: &IntentId) -> Option<&dyn IntentHandler> {
        self.handlers.get(intent).map(Arc::as_ref)
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
        let registry = IntentHandlerRegistry::new(vec![Arc::new(StubHandler)]);

        assert!(registry.get(&IntentId::AskOpeningHours).is_some());
        assert!(registry.get(&IntentId::Greeting).is_none());
    }

    #[test]
    fn registry_resolves_known_intent_policy() {
        let registry = IntentHandlerRegistry::new(vec![Arc::new(StubHandler)]);

        let policy = registry.find_policy(&IntentId::AskOpeningHours).unwrap();

        assert_eq!(policy.id, IntentId::AskOpeningHours);
    }
}
