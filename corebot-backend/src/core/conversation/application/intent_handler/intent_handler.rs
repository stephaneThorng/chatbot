use std::collections::HashMap;

use crate::core::conversation::application::dto::nlu_analysis_result::NluEntityResult;
use crate::core::conversation::domain::conversation::Conversation;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, WorkflowConfig};
use crate::core::conversation::domain::model::slot::{
    SlotConstraint, SlotDataType, SlotDataValue, SlotName,
};
use crate::core::conversation::domain::workflow::NextSlot;
use chrono::Local;
use rust_i18n::t;

/// Stateless input passed to an intent-specific handler.
pub struct IntentHandlerInput<'a> {
    pub conversation: Conversation,
    pub analysis_intent: &'a IntentId,
    pub text: &'a str,
    pub analysis_entities: &'a [NluEntityResult],
}

/// Immediate result produced by an intent handler.
pub struct StateHandlerResult {
    pub updated_conversation: Conversation,
    pub reply: Vec<String>,
    pub handled_intent: IntentId,
}

pub enum WorkflowPostProcessResult {
    Succeeded {
        updated_conversation: Conversation,
        reply: Option<Vec<String>>,
    },
    Failed {
        updated_conversation: Conversation,
        reply: Vec<String>,
    },
}

/// Stateless application component that handles one intent.
#[async_trait::async_trait]
pub trait IntentHandler: Send + Sync {
    fn intent(&self) -> IntentId;
    fn config(&self) -> IntentConfig;

    fn is_workflow(&self) -> bool {
        self.config().workflow.is_workflow()
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult;

    fn lookup_entity_value<'a>(
        &self,
        input: &'a IntentHandlerInput<'a>,
        entity_label: &str,
    ) -> Option<&'a str> {
        input
            .analysis_entities
            .iter()
            .find(|entity| entity.entity_label == entity_label)
            .map(|entity| entity.value.as_str())
    }

    async fn handle_workflow(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let config = self.config();
        let workflow_cfg = config.workflow.workflow_config();
        let (mut updated_conversation, just_started) =
            match self.start_or_cancel_workflow(&input, &config) {
                Ok(state) => state,
                Err(result) => return result,
            };

        let (next_conversation, applied_updates) =
            match self.collect_workflow_updates(&input, workflow_cfg, updated_conversation) {
                Ok(state) => state,
                Err(result) => return result,
            };
        updated_conversation = next_conversation;
        if !self.is_ready_for_confirmation(&updated_conversation) {
            return self.build_collection_reply(
                workflow_cfg,
                updated_conversation,
                just_started,
                &applied_updates,
            );
        }

        self.build_confirmation_reply(
            workflow_cfg,
            input.analysis_intent,
            updated_conversation,
            &applied_updates,
        )
        .await
    }

    fn start_or_cancel_workflow(
        &self,
        input: &IntentHandlerInput<'_>,
        config: &IntentConfig,
    ) -> Result<(Conversation, bool), StateHandlerResult> {
        let mut updated_conversation = input.conversation.clone();
        let just_started = !updated_conversation.has_active_workflow();

        if just_started {
            updated_conversation = updated_conversation
                .into_started_workflow(config)
                .expect("workflow handler config must start a workflow");
        }

        if input.analysis_intent == &IntentId::Cancel {
            updated_conversation = updated_conversation.into_cancelled_workflow();
            return Err(StateHandlerResult {
                reply: vec![
                    t!(
                        "system.workflow_cancelled",
                        locale = updated_conversation.lang.as_str()
                    )
                    .to_string(),
                ],
                updated_conversation,
                handled_intent: self.intent(),
            });
        }

        Ok((updated_conversation, just_started))
    }

    fn collect_workflow_updates(
        &self,
        input: &IntentHandlerInput<'_>,
        workflow_cfg: &WorkflowConfig,
        conversation: Conversation,
    ) -> Result<(Conversation, Vec<SlotUpdate>), StateHandlerResult> {
        let extracted_updates =
            self.extract_slot_updates(&conversation, input.text, input.analysis_entities);
        let update_result = self.apply_slot_updates(conversation, extracted_updates);
        let mut updated_conversation = update_result.updated_conversation;

        if let Some(invalid_slot) = update_result.invalid_slot {
            return Err(StateHandlerResult {
                reply: vec![self.slot_prompt(
                    workflow_cfg,
                    invalid_slot,
                    updated_conversation.lang.as_str(),
                )],
                updated_conversation,
                handled_intent: self.intent(),
            });
        }

        if let Some(reply) = self.validate_constraints(workflow_cfg, &mut updated_conversation) {
            return Err(StateHandlerResult {
                updated_conversation,
                reply: vec![reply],
                handled_intent: self.intent(),
            });
        }

        Ok((updated_conversation, update_result.applied_updates))
    }

    fn is_ready_for_confirmation(&self, conversation: &Conversation) -> bool {
        conversation
            .active_workflow()
            .is_some_and(|workflow| workflow.is_ready_for_confirmation())
    }

    fn build_collection_reply(
        &self,
        workflow_cfg: &WorkflowConfig,
        updated_conversation: Conversation,
        just_started: bool,
        applied_updates: &[SlotUpdate],
    ) -> StateHandlerResult {
        let slot_prompt = self.next_slot_prompt(&updated_conversation, workflow_cfg);
        let reply = if just_started {
            vec![self.starting_reply(
                workflow_cfg,
                updated_conversation.lang.as_str(),
                &slot_prompt,
            )]
        } else if applied_updates.is_empty() {
            vec![
                self.workflow_misunderstanding_reply(updated_conversation.lang.as_str()),
                slot_prompt,
            ]
        } else {
            vec![
                self.workflow_acknowledgement_reply(
                    updated_conversation.lang.as_str(),
                    applied_updates,
                ),
                slot_prompt,
            ]
        };

        StateHandlerResult {
            updated_conversation,
            reply,
            handled_intent: self.intent(),
        }
    }

    async fn build_confirmation_reply(
        &self,
        workflow_cfg: &WorkflowConfig,
        analysis_intent: &IntentId,
        updated_conversation: Conversation,
        applied_updates: &[SlotUpdate],
    ) -> StateHandlerResult {
        let updated_any_slot = !applied_updates.is_empty();
        match analysis_intent {
            IntentId::Affirmative if !updated_any_slot => {
                self.confirm_workflow(workflow_cfg, updated_conversation)
                    .await
            }
            IntentId::Negative if !updated_any_slot => StateHandlerResult {
                reply: vec![self.negative_prompt(updated_conversation.lang.as_str())],
                updated_conversation,
                handled_intent: self.intent(),
            },
            _ => StateHandlerResult {
                reply: if updated_any_slot {
                    vec![
                        self.workflow_acknowledgement_reply(
                            updated_conversation.lang.as_str(),
                            applied_updates,
                        ),
                        self.confirmation_prompt(workflow_cfg, &updated_conversation),
                    ]
                } else {
                    vec![
                        self.workflow_misunderstanding_reply(updated_conversation.lang.as_str()),
                        self.confirmation_prompt(workflow_cfg, &updated_conversation),
                    ]
                },
                updated_conversation,
                handled_intent: self.intent(),
            },
        }
    }

    async fn confirm_workflow(
        &self,
        workflow_cfg: &WorkflowConfig,
        mut updated_conversation: Conversation,
    ) -> StateHandlerResult {
        let lang = updated_conversation.lang.clone();
        updated_conversation = updated_conversation.into_confirmed_workflow();

        match self.post_process(lang.as_str(), updated_conversation).await {
            WorkflowPostProcessResult::Succeeded {
                mut updated_conversation,
                reply,
            } => {
                let lang = updated_conversation.lang.clone();
                updated_conversation = updated_conversation.into_completed_workflow();
                StateHandlerResult {
                    reply: reply.unwrap_or_else(|| {
                        vec![self.completion_reply(workflow_cfg, lang.as_str())]
                    }),
                    updated_conversation,
                    handled_intent: self.intent(),
                }
            }
            WorkflowPostProcessResult::Failed {
                updated_conversation,
                reply,
            } => StateHandlerResult {
                updated_conversation: updated_conversation.into_reopened_workflow(),
                reply,
                handled_intent: self.intent(),
            },
        }
    }

    async fn post_process(
        &self,
        _lang: &str,
        conversation: Conversation,
    ) -> WorkflowPostProcessResult {
        WorkflowPostProcessResult::Succeeded {
            updated_conversation: conversation,
            reply: None,
        }
    }

    fn negative_prompt(&self, lang: &str) -> String {
        t!("system.workflow_update_prompt", locale = lang).to_string()
    }

    fn next_slot_prompt(
        &self,
        conversation: &Conversation,
        workflow_cfg: &WorkflowConfig,
    ) -> String {
        let Some(workflow) = conversation.active_workflow() else {
            return self.confirmation_prompt(workflow_cfg, conversation);
        };
        match workflow.next_required_slot() {
            Some(NextSlot::Data(definition)) => {
                self.slot_prompt(workflow_cfg, definition.name, conversation.lang.as_str())
            }
            Some(NextSlot::Confirmation) | None => {
                self.confirmation_prompt(workflow_cfg, conversation)
            }
        }
    }

    fn slot_prompt(
        &self,
        workflow_cfg: &WorkflowConfig,
        slot_name: SlotName,
        lang: &str,
    ) -> String {
        workflow_cfg
            .slots
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

    fn confirmation_prompt(
        &self,
        workflow_cfg: &WorkflowConfig,
        conversation: &Conversation,
    ) -> String {
        workflow_cfg
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

    fn completion_reply(&self, workflow_cfg: &WorkflowConfig, lang: &str) -> String {
        workflow_cfg
            .completion_response
            .as_ref()
            .map(|key| t!(key.0.as_str(), locale = lang).to_string())
            .unwrap_or_else(|| t!("system.workflow_complete", locale = lang).to_string())
    }

    fn starting_reply(
        &self,
        workflow_cfg: &WorkflowConfig,
        lang: &str,
        slot_prompt: &str,
    ) -> String {
        match workflow_cfg.starting_message.as_ref() {
            Some(key) => {
                let starting = t!(key.0.as_str(), locale = lang).to_string();
                format!("{}\n{}", starting, slot_prompt)
            }
            None => slot_prompt.to_string(),
        }
    }

    /// Evaluate all declarative constraints on currently filled slots.
    fn validate_constraints(
        &self,
        workflow_cfg: &WorkflowConfig,
        conversation: &mut Conversation,
    ) -> Option<String> {
        let lang = conversation.lang.clone();
        for slot_def in &workflow_cfg.slots {
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

    fn check_slot_constraint(
        &self,
        constraint: &SlotConstraint,
        value: &SlotDataValue,
    ) -> Option<&'static str> {
        match (constraint, value) {
            (SlotConstraint::TextMaxLen(max), SlotDataValue::Text(s)) => {
                if s.len() > *max {
                    Some(SlotConstraint::TextMaxLen(0).default_error_key())
                } else {
                    None
                }
            }
            (SlotConstraint::EmailFormat, SlotDataValue::Text(s)) => {
                let valid = s.contains('@')
                    && s.split('@').nth(0).is_some_and(|local| !local.is_empty())
                    && s.split('@')
                        .nth(1)
                        .is_some_and(|domain| domain.contains('.') && domain.len() > 2);
                if valid {
                    None
                } else {
                    Some(SlotConstraint::EmailFormat.default_error_key())
                }
            }
            (SlotConstraint::NumberRange(min, max), SlotDataValue::Number(n)) => {
                if n < min || n > max {
                    Some(SlotConstraint::NumberRange(0, 0).default_error_key())
                } else {
                    None
                }
            }
            (SlotConstraint::FutureDate, SlotDataValue::Date(date)) => {
                let today = Local::now().date_naive();
                if *date < today {
                    Some(SlotConstraint::FutureDate.default_error_key())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn extract_slot_updates(
        &self,
        conversation: &Conversation,
        raw_text: &str,
        entities: &[NluEntityResult],
    ) -> SlotUpdateResult {
        let Some(workflow) = conversation.active_workflow() else {
            return SlotUpdateResult { updates: vec![] };
        };

        let mut updates = vec![];

        if workflow.is_ready_for_confirmation() {
            for slot_cfg in workflow.slot_definitions() {
                let matching_entities = entities
                    .iter()
                    .filter(|entity| {
                        slot_cfg
                            .name
                            .entity_type_labels()
                            .contains(&entity.entity_label.as_str())
                    })
                    .collect::<Vec<_>>();

                if slot_cfg.name == SlotName::People && matching_entities.len() > 1 {
                    if let Some(value) =
                        self.sum_numeric_entities(&matching_entities, conversation.lang.as_str())
                    {
                        updates.push(SlotUpdate {
                            slot_name: slot_cfg.name,
                            value: SlotDataValue::Number(value),
                        });
                    }
                    continue;
                }

                for entity in matching_entities {
                    let Some(slot_value) = self.slot_value_from_name(
                        slot_cfg.name,
                        entity.value.as_str(),
                        conversation.lang.as_str(),
                    ) else {
                        continue;
                    };
                    updates.push(SlotUpdate {
                        slot_name: slot_cfg.name,
                        value: slot_value,
                    });
                }
            }
        } else {
            for slot_cfg in workflow.slot_definitions() {
                let matching_entities = entities
                    .iter()
                    .filter(|entity| {
                        slot_cfg
                            .name
                            .entity_type_labels()
                            .contains(&entity.entity_label.as_str())
                    })
                    .collect::<Vec<_>>();

                let matched_update = if slot_cfg.name == SlotName::People
                    && matching_entities.len() > 1
                {
                    self.sum_numeric_entities(&matching_entities, conversation.lang.as_str())
                        .map(|value| SlotUpdate {
                            slot_name: slot_cfg.name,
                            value: SlotDataValue::Number(value),
                        })
                } else {
                    matching_entities.into_iter().find_map(|entity| {
                        self.slot_value_from_name(
                            slot_cfg.name,
                            entity.value.as_str(),
                            conversation.lang.as_str(),
                        )
                        .map(|value| SlotUpdate {
                            slot_name: slot_cfg.name,
                            value,
                        })
                    })
                };

                if let Some(update) = matched_update {
                    updates.push(update);
                    continue;
                }

                if workflow.slot_value(slot_cfg.name).is_none() {
                    break;
                }
            }
        }

        if updates.is_empty()
            && let Some(NextSlot::Data(next_slot)) = workflow.next_required_slot()
            && next_slot.name.data_type() == SlotDataType::Number
            && let Some(number) = self.parse_bare_number_reply(raw_text, conversation.lang.as_str())
        {
            updates.push(SlotUpdate {
                slot_name: next_slot.name,
                value: SlotDataValue::Number(number),
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
        let mut applied_updates = vec![];
        let mut invalid_slot = None;

        for update in extracted_updates.updates {
            if let Err(error) =
                updated_conversation.set_workflow_slot(update.slot_name, update.value.clone())
            {
                invalid_slot = Some(error.slot);
                break;
            }
            applied_updates.push(update);
        }

        SlotUpdateApplicationResult {
            updated_conversation,
            applied_updates,
            invalid_slot,
        }
    }

    fn workflow_misunderstanding_reply(&self, lang: &str) -> String {
        t!("system.workflow_not_understood", locale = lang).to_string()
    }

    fn workflow_acknowledgement_reply(&self, lang: &str, updates: &[SlotUpdate]) -> String {
        let values = updates
            .iter()
            .map(|update| {
                format!(
                    "{} '{}'",
                    update.slot_name.as_str(),
                    self.format_slot_value(&update.value, lang)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        t!(
            "system.workflow_slot_acknowledged",
            locale = lang,
            values = values
        )
        .to_string()
    }

    fn format_slot_value(&self, value: &SlotDataValue, lang: &str) -> String {
        match value {
            SlotDataValue::Text(text) => text.clone(),
            SlotDataValue::Date(date) => date
                .format(if lang == "id" { "%d %B %Y" } else { "%B %d %Y" })
                .to_string(),
            SlotDataValue::Time(time) => time.format("%H:%M").to_string(),
            SlotDataValue::Number(number) => number.to_string(),
            SlotDataValue::Boolean(flag) => flag.to_string(),
        }
    }

    fn slot_value_from_name(
        &self,
        slot_name: SlotName,
        raw_value: &str,
        lang: &str,
    ) -> Option<SlotDataValue> {
        SlotDataValue::from_text(slot_name.data_type(), raw_value, lang)
    }

    fn parse_bare_number_reply(&self, raw_value: &str, lang: &str) -> Option<u32> {
        let normalized = raw_value.trim();
        if normalized.is_empty() {
            return None;
        }

        if let Some(SlotDataValue::Number(number)) =
            SlotDataValue::from_text(SlotDataType::Number, normalized, lang)
        {
            return Some(number);
        }
        None
    }

    fn sum_numeric_entities(&self, entities: &[&NluEntityResult], lang: &str) -> Option<u32> {
        let mut total = 0_u32;
        for entity in entities {
            let SlotDataValue::Number(value) =
                SlotDataValue::from_text(SlotDataType::Number, entity.value.as_str(), lang)?
            else {
                return None;
            };
            total = total.checked_add(value)?;
        }
        Some(total)
    }
}

pub struct SlotUpdateResult {
    pub updates: Vec<SlotUpdate>,
}

pub struct SlotUpdateApplicationResult {
    pub updated_conversation: Conversation,
    pub applied_updates: Vec<SlotUpdate>,
    pub invalid_slot: Option<SlotName>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlotUpdate {
    pub slot_name: SlotName,
    pub value: SlotDataValue,
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

    pub fn get(&self, intent: &IntentId) -> Option<&dyn IntentHandler> {
        self.handlers.get(intent).map(Box::as_ref)
    }

    pub fn find_config(&self, intent: &IntentId) -> Option<IntentConfig> {
        self.get(intent).map(IntentHandler::config)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveTime;

    use super::*;
    use crate::core::conversation::domain::model::domain_type::DomainType;
    use crate::core::conversation::domain::model::intent::{IntentWorkflow, WorkflowConfig};
    use crate::core::conversation::domain::model::slot::SlotConfig;
    use crate::core::conversation::domain::service::date_resolver::resolve_date;

    struct StubHandler;

    #[async_trait::async_trait]
    impl IntentHandler for StubHandler {
        fn intent(&self) -> IntentId {
            IntentId::AskOpeningHours
        }

        fn config(&self) -> IntentConfig {
            IntentConfig {
                id: self.intent(),
                workflow: IntentWorkflow::Informational,
            }
        }

        async fn handle(&self, _input: IntentHandlerInput<'_>) -> StateHandlerResult {
            StateHandlerResult {
                updated_conversation: Conversation::new(DomainType::Restaurant),
                reply: vec!["handled".to_string()],
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
    fn registry_resolves_known_intent_config() {
        let registry = IntentHandlerRegistry::new(vec![Box::new(StubHandler)]);
        let config = registry.find_config(&IntentId::AskOpeningHours).unwrap();
        assert_eq!(config.id, IntentId::AskOpeningHours);
    }

    struct StubWorkflowHandler;

    #[async_trait::async_trait]
    impl IntentHandler for StubWorkflowHandler {
        fn intent(&self) -> IntentId {
            IntentId::ReservationCreate
        }

        fn config(&self) -> IntentConfig {
            IntentConfig {
                id: self.intent(),
                workflow: IntentWorkflow::Workflow(WorkflowConfig {
                    nlu_task: None,
                    slots: vec![
                        SlotConfig {
                            name: SlotName::Name,
                            required: true,
                            prompt: crate::core::conversation::domain::model::intent::i18n_key(
                                "test.name",
                            ),
                            constraints: vec![],
                        },
                        SlotConfig {
                            name: SlotName::Date,
                            required: true,
                            prompt: crate::core::conversation::domain::model::intent::i18n_key(
                                "test.date",
                            ),
                            constraints: vec![],
                        },
                        SlotConfig {
                            name: SlotName::Time,
                            required: true,
                            prompt: crate::core::conversation::domain::model::intent::i18n_key(
                                "test.time",
                            ),
                            constraints: vec![],
                        },
                        SlotConfig {
                            name: SlotName::People,
                            required: true,
                            prompt: crate::core::conversation::domain::model::intent::i18n_key(
                                "test.people",
                            ),
                            constraints: vec![],
                        },
                    ],
                    starting_message: None,
                    confirmation_prompt: None,
                    completion_response: None,
                }),
            }
        }

        async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
            self.handle_workflow(input).await
        }
    }

    impl StubWorkflowHandler {
        fn handle_blocking(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("test runtime should be created")
                .block_on(<Self as IntentHandler>::handle(self, input))
        }
    }

    fn workflow_handler() -> StubWorkflowHandler {
        StubWorkflowHandler
    }

    fn workflow_entity(entity_label: &'static str, value: &str) -> NluEntityResult {
        NluEntityResult {
            entity_label: entity_label.to_string(),
            value: value.to_string(),
            raw_value: value.to_string(),
            start: 0,
            end: value.len(),
            confidence: 1.0,
        }
    }

    #[test]
    fn workflow_entity_updates_stop_at_first_missing_slot() {
        let conversation = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_handler().config())
            .unwrap()
            .into_workflow_slot(SlotName::Name, SlotDataValue::Text("Alice".to_string()))
            .unwrap();

        let result = workflow_handler().handle_blocking(IntentHandlerInput {
            conversation,
            analysis_intent: &IntentId::ReservationCreate,
            text: "in 2 days",
            analysis_entities: &[workflow_entity("people_count", "2")],
        });

        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(workflow.slot_value(SlotName::People), None);
        assert_eq!(workflow.slot_value(SlotName::Date), None);
    }

    #[test]
    fn workflow_sequential_entities_can_fill_current_and_following_slots() {
        let conversation = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_handler().config())
            .unwrap()
            .into_workflow_slot(SlotName::Name, SlotDataValue::Text("Alice".to_string()))
            .unwrap();

        let result = workflow_handler().handle_blocking(IntentHandlerInput {
            conversation,
            analysis_intent: &IntentId::ReservationCreate,
            text: "tomorrow at 7pm for 4 people",
            analysis_entities: &[
                workflow_entity("date", "tomorrow"),
                workflow_entity("time", "7pm"),
                workflow_entity("people_count", "4"),
            ],
        });

        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slot_value(SlotName::Date),
            Some(&SlotDataValue::Date(resolve_date("tomorrow").unwrap()))
        );
        assert_eq!(
            workflow.slot_value(SlotName::Time),
            Some(&SlotDataValue::Time(
                NaiveTime::from_hms_opt(19, 0, 0).unwrap()
            ))
        );
        assert_eq!(
            workflow.slot_value(SlotName::People),
            Some(&SlotDataValue::Number(4))
        );
    }

    #[test]
    fn workflow_entity_number_words_fill_numeric_slot() {
        let conversation = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_handler().config())
            .unwrap()
            .into_workflow_slot(SlotName::Name, SlotDataValue::Text("Alice".to_string()))
            .unwrap()
            .into_workflow_slot(
                SlotName::Date,
                SlotDataValue::Date(resolve_date("tomorrow").unwrap()),
            )
            .unwrap()
            .into_workflow_slot(
                SlotName::Time,
                SlotDataValue::Time(NaiveTime::from_hms_opt(19, 0, 0).unwrap()),
            )
            .unwrap();

        let result = workflow_handler().handle_blocking(IntentHandlerInput {
            conversation,
            analysis_intent: &IntentId::ReservationCreate,
            text: "six",
            analysis_entities: &[workflow_entity("people_count", "six")],
        });

        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slot_value(SlotName::People),
            Some(&SlotDataValue::Number(6))
        );
    }

    #[test]
    fn workflow_multiple_people_entities_are_summed() {
        let conversation = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&workflow_handler().config())
            .unwrap()
            .into_workflow_slot(SlotName::Name, SlotDataValue::Text("Alice".to_string()))
            .unwrap()
            .into_workflow_slot(
                SlotName::Date,
                SlotDataValue::Date(resolve_date("tomorrow").unwrap()),
            )
            .unwrap()
            .into_workflow_slot(
                SlotName::Time,
                SlotDataValue::Time(NaiveTime::from_hms_opt(19, 0, 0).unwrap()),
            )
            .unwrap();

        let result = workflow_handler().handle_blocking(IntentHandlerInput {
            conversation,
            analysis_intent: &IntentId::ReservationCreate,
            text: "2 adults and 1 child",
            analysis_entities: &[
                workflow_entity("people_count", "2"),
                workflow_entity("people_count", "1"),
            ],
        });

        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slot_value(SlotName::People),
            Some(&SlotDataValue::Number(3))
        );
    }
}
