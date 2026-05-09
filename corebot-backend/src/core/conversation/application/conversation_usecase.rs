use std::str::FromStr;
use std::sync::LazyLock;
use std::sync::Arc;

use langdetect_rs::detector_factory::DetectorFactory;
use rust_i18n::t;

use super::conversation_command::{HandleConversationCommand, HandleConversationResult};
use super::port::inbound::conversation_trait::HandleConversationPort;
use super::port::outbound::conversation_repository::ConversationRepositoryPort;
use super::port::outbound::domain_gateway_trait::DomainGatewayPort;
use super::port::outbound::nlp_analyzer_trait::NlpEngineGatewayPort;
use crate::core::conversation::domain::conversation::Conversation;
use crate::core::conversation::domain::conversation_id::ConversationId;
use crate::core::conversation::domain::domain_type::DomainType;
use crate::core::conversation::domain::intent::{IntentCatalog, IntentId, IntentResponse, NluTask, SlotDefinition, build_catalog};
use crate::core::conversation::domain::slot::{SlotType, SlotValue};
use crate::core::conversation::domain::workflow::NextSlot;
use crate::core::nlu_engine::domain::analysis::{NluAnalysis, NluEntity};

static LANGUAGE_DETECTOR: LazyLock<DetectorFactory> =
    LazyLock::new(|| DetectorFactory::default().build());

pub struct HandleConversationUseCase {
    domain: DomainType,
    domain_gateway: Arc<dyn DomainGatewayPort>,
    nlu_engine_gateway: Arc<dyn NlpEngineGatewayPort>,
    conversation_repository: Arc<dyn ConversationRepositoryPort>,
}

impl HandleConversationPort for HandleConversationUseCase {
    fn handle_message(&self, command: HandleConversationCommand) -> HandleConversationResult {
        let (mut conversation, session_id) =
            self.load_or_create_conversation(command.session_id.as_deref(), &command.message);
        let catalog = build_catalog(conversation.domain);
        let analysis = self.analyze_message(&conversation, &catalog, &command.message);

        let reply = if conversation.has_active_workflow() {
            self.handle_active_workflow(&mut conversation, &analysis, &catalog)
        } else {
            self.handle_idle_intent(&mut conversation, &analysis, &catalog)
        };

        self.save_conversation(&conversation);

        HandleConversationResult { session_id, reply }
    }
}

impl HandleConversationUseCase {
    pub fn new(
        domain: DomainType,
        domain_gateway: Arc<dyn DomainGatewayPort>,
        nlu_engine_gateway: Arc<dyn NlpEngineGatewayPort>,
        conversation_repository: Arc<dyn ConversationRepositoryPort>,
    ) -> Self {
        Self {
            domain,
            domain_gateway,
            nlu_engine_gateway,
            conversation_repository,
        }
    }

    fn load_or_create_conversation(&self, session_id: Option<&str>, message: &str) -> (Conversation, String) {
        let parsed_id = session_id.and_then(|id| ConversationId::from_str(id).ok());
        if let Some(conversation_id) = parsed_id {
            if let Ok(Some(conversation)) = self.conversation_repository.load(&conversation_id) {
                return (conversation, conversation_id.to_string());
            }

            let mut conversation = Conversation::with_id(conversation_id.clone(), self.domain);
            conversation.lang = Self::detect_language(message).to_string();
            return (conversation, conversation_id.to_string());
        }

        let mut conversation = Conversation::new(self.domain);
        conversation.lang = Self::detect_language(message).to_string();
        let session_id = conversation.id.to_string();
        (conversation, session_id)
    }

    fn detect_task(conversation: &Conversation, catalog: &IntentCatalog) -> Option<String> {
        let workflow = conversation.active_workflow()?;
        if workflow.is_ready_for_confirmation() {
            return Some(NluTask::Choice.as_tag().to_string());
        }

        catalog
            .nlu_task(&workflow.intent)
            .map(|task| task.as_tag().to_string())
    }

    fn analyze_message(&self, conversation: &Conversation, catalog: &IntentCatalog, message: &str) -> NluAnalysis {
        self.nlu_engine_gateway
            .analyze(
                message,
                &conversation.lang,
                Self::domain_tag(conversation.domain),
                Self::detect_task(conversation, catalog),
            )
    }

    fn save_conversation(&self, conversation: &Conversation) {
        let _ = self.conversation_repository.save(conversation);
    }

    fn handle_idle_intent(
        &self,
        conversation: &mut Conversation,
        analysis: &NluAnalysis,
        catalog: &IntentCatalog,
    ) -> String {
        let intent = IntentId::new(&analysis.intent.name);
        if catalog.is_workflow(&intent) {
            let _ = conversation.start_workflow(&intent, catalog);
            self.fill_slots_from_entities(conversation, analysis, catalog);
            return self.reply_for_workflow_state(conversation, catalog);
        }

        let Some(policy) = catalog.get(&intent) else {
            return self.translate_system_text(catalog, "echo_intent", &conversation.lang, "intent", &analysis.intent.name);
        };

        match &policy.response {
            IntentResponse::DomainOpeningHours => self.domain_gateway.get_opening_hours(),
            IntentResponse::Static(key) => self.translate_key(&key.0, &conversation.lang),
            IntentResponse::EchoIntent => {
                if analysis.intent.name == "cancel" {
                    self.translate_system_text(catalog, "no_active_workflow_to_cancel", &conversation.lang, "", "")
                } else {
                    self.translate_system_text(catalog, "echo_intent", &conversation.lang, "intent", &analysis.intent.name)
                }
            }
        }
    }

    fn handle_active_workflow(
        &self,
        conversation: &mut Conversation,
        analysis: &NluAnalysis,
        catalog: &IntentCatalog,
    ) -> String {
        if analysis.intent.name == "cancel" {
            conversation.cancel_workflow();
            return self.translate_system_text(catalog, "workflow_cancelled", &conversation.lang, "", "");
        }

        if let Some(workflow) = conversation.active_workflow() {
            if workflow.is_ready_for_confirmation() {
                return self.handle_confirmation_step(conversation, &analysis.intent.name, catalog);
            }
        }

        self.fill_slots_from_entities(conversation, analysis, catalog);
        self.reply_for_workflow_state(conversation, catalog)
    }

    fn handle_confirmation_step(
        &self,
        conversation: &mut Conversation,
        intent_name: &str,
        catalog: &IntentCatalog,
    ) -> String {
        match intent_name {
            "affirmative" => {
                if let Some(workflow) = conversation.active_workflow_mut() {
                    let _ = workflow.fill_slot("confirmation", SlotValue::Boolean(true));
                    let completed_intent = workflow.intent.clone();
                    conversation.complete_workflow();
                    return catalog
                        .completion_response_key(&completed_intent)
                        .map(|key| self.translate_key(key, &conversation.lang))
                        .unwrap_or_else(|| {
                            self.translate_system_text(catalog, "workflow_complete", &conversation.lang, "", "")
                        });
                }
                self.translate_system_text(catalog, "no_active_workflow", &conversation.lang, "", "")
            }
            "negative" => {
                conversation.cancel_workflow();
                self.translate_system_text(catalog, "workflow_cancelled", &conversation.lang, "", "")
            }
            _ => self.translate_system_text(catalog, "confirm_yes_no", &conversation.lang, "", ""),
        }
    }

    fn fill_slots_from_entities(
        &self,
        conversation: &mut Conversation,
        analysis: &NluAnalysis,
        catalog: &IntentCatalog,
    ) {
        let Some(workflow) = conversation.active_workflow_mut() else {
            return;
        };
        let slot_definitions = catalog.required_slots(&workflow.intent);

        for entity in &analysis.entities {
            for slot in &slot_definitions {
                if !slot.entity_types.iter().any(|entity_type| entity_type == &entity.entity_type) {
                    continue;
                }
                if let Some(slot_value) = Self::slot_value_from_entity(slot, entity) {
                    let _ = workflow.fill_slot(&slot.name, slot_value);
                }
            }
        }
    }

    fn parse_people_count(entity: &NluEntity) -> Option<u32> {
        let digits = entity
            .value
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>();
        digits.parse().ok()
    }

    fn slot_value_from_entity(slot: &SlotDefinition, entity: &NluEntity) -> Option<SlotValue> {
        match slot.slot_type {
            SlotType::Text => Some(SlotValue::Text(entity.value.clone())),
            SlotType::Date => Some(SlotValue::Date(entity.value.clone())),
            SlotType::Time => Some(SlotValue::Time(entity.value.clone())),
            SlotType::Number => Self::parse_people_count(entity).map(SlotValue::Number),
            SlotType::Boolean => None,
        }
    }

    fn reply_for_workflow_state(&self, conversation: &Conversation, catalog: &IntentCatalog) -> String {
        let Some(workflow) = conversation.active_workflow() else {
            return self.translate_system_text(catalog, "no_active_workflow", &conversation.lang, "", "");
        };

        match workflow.next_required_slot() {
            Some(NextSlot::Data(def)) => catalog
                .slot_prompt_key(&workflow.intent, &def.name)
                .map(|key| self.translate_key(key, &conversation.lang))
                .unwrap_or_else(|| {
                    self.translate_system_text(
                        catalog,
                        "missing_slot_fallback",
                        &conversation.lang,
                        "slot",
                        &def.name,
                    )
                }),
            Some(NextSlot::Confirmation) => catalog
                .confirmation_prompt_key(&workflow.intent)
                .map(|key| self.translate_key(key, &conversation.lang))
                .unwrap_or_else(|| self.translate_system_text(catalog, "confirm_generic", &conversation.lang, "", "")),
            None => self.translate_system_text(catalog, "workflow_complete", &conversation.lang, "", ""),
        }
    }

    fn domain_tag(domain: DomainType) -> &'static str {
        match domain {
            DomainType::Restaurant => "restaurant",
            DomainType::Hotel => "hotel",
        }
    }

    fn detect_language(message: &str) -> &'static str {
        match LANGUAGE_DETECTOR.detect(message, None).ok().as_deref() {
            Some("id") => "id",
            Some("en") => "en",
            _ => "en",
        }
    }

    fn translate_key(&self, key: &str, lang: &str) -> String {
        t!(key, locale = lang).to_string()
    }

    fn translate_system_text(
        &self,
        catalog: &IntentCatalog,
        system_key: &str,
        lang: &str,
        arg_key: &str,
        arg_value: &str,
    ) -> String {
        let Some(i18n_key) = catalog.system_text_key(system_key) else {
            return String::new();
        };
        if arg_key.is_empty() {
            return self.translate_key(i18n_key, lang);
        }
        match arg_key {
            "intent" => t!(i18n_key, locale = lang, intent = arg_value).to_string(),
            "slot" => t!(i18n_key, locale = lang, slot = arg_value).to_string(),
            _ => self.translate_key(i18n_key, lang),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;

    use crate::core::conversation::application::port::outbound::conversation_repository::RepositoryError;
    use crate::core::nlu_engine::domain::analysis::{NluIntent, NerTokenLabel, NluIntentCandidate};

    struct StubDomainGateway;

    impl DomainGatewayPort for StubDomainGateway {
        fn get_opening_hours(&self) -> String {
            "Mon-Sun 9am-10pm".to_string()
        }
    }

    struct StubConversationRepository {
        store: RwLock<HashMap<ConversationId, Conversation>>,
    }

    impl StubConversationRepository {
        fn new() -> Self {
            Self {
                store: RwLock::new(HashMap::new()),
            }
        }
    }

    impl ConversationRepositoryPort for StubConversationRepository {
        fn save(&self, conversation: &Conversation) -> Result<(), RepositoryError> {
            self.store
                .write()
                .unwrap()
                .insert(conversation.id.clone(), conversation.clone());
            Ok(())
        }

        fn load(&self, id: &ConversationId) -> Result<Option<Conversation>, RepositoryError> {
            Ok(self.store.read().unwrap().get(id).cloned())
        }

        fn delete(&self, id: &ConversationId) -> Result<(), RepositoryError> {
            self.store.write().unwrap().remove(id);
            Ok(())
        }
    }

    struct StubNlpAnalyzer {
        responses: std::sync::Mutex<Vec<NluAnalysis>>,
        tasks: std::sync::Mutex<Vec<Option<String>>>,
    }

    impl StubNlpAnalyzer {
        fn new(responses: Vec<NluAnalysis>) -> Self {
            Self {
                responses: std::sync::Mutex::new(responses.into_iter().rev().collect()),
                tasks: std::sync::Mutex::new(vec![]),
            }
        }

        fn recorded_tasks(&self) -> Vec<Option<String>> {
            self.tasks.lock().unwrap().clone()
        }
    }

    impl NlpEngineGatewayPort for StubNlpAnalyzer {
        fn analyze(
            &self,
            text: &str,
            lang: &str,
            domain: &str,
            task: Option<String>,
        ) -> NluAnalysis {
            let _ = (text, lang, domain);
            self.tasks.lock().unwrap().push(task);
            self.responses
                .lock()
                .unwrap()
                .pop()
                .expect("missing stub NLU response")
        }
    }

    fn analysis(intent_name: &'static str, entities: Vec<NluEntity>) -> NluAnalysis {
        NluAnalysis {
            tagged_text: String::new(),
            intent: NluIntent {
                name: intent_name.to_string(),
                confidence: 1.0,
            },
            intents: Vec::<NluIntentCandidate>::new(),
            entities,
            ner_labels: Vec::<NerTokenLabel>::new(),
        }
    }

    fn entity(entity_type: &str, value: &str) -> NluEntity {
        NluEntity {
            entity_type: entity_type.to_string(),
            value: value.to_string(),
            raw_value: value.to_string(),
            start: 0,
            end: value.len(),
            confidence: 1.0,
        }
    }

    fn make_use_case(analyzer: Arc<StubNlpAnalyzer>) -> HandleConversationUseCase {
        let repo: Arc<dyn ConversationRepositoryPort> = Arc::new(StubConversationRepository::new());
        HandleConversationUseCase::new(
            DomainType::Restaurant,
            Arc::new(StubDomainGateway),
            analyzer,
            repo,
        )
    }

    fn make_command(message: &str, session_id: Option<&str>) -> HandleConversationCommand {
        HandleConversationCommand {
            message: message.to_string(),
            session_id: session_id.map(str::to_string),
        }
    }

    #[test]
    fn handle_message_reuses_provided_session_id() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![analysis("greeting", vec![])]));
        let result = make_use_case(analyzer).handle_message(make_command("hello", Some(&ConversationId::new().to_string())));
        assert!(!result.session_id.is_empty());
    }

    #[test]
    fn handle_message_generates_session_id_when_none() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![analysis("greeting", vec![])]));
        let result = make_use_case(analyzer).handle_message(make_command("hello", None));
        assert!(!result.session_id.is_empty());
    }

    #[test]
    fn handle_message_delegates_opening_hours_reply_to_domain_gateway() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![analysis("ask_opening_hours", vec![])]));
        let result = make_use_case(analyzer).handle_message(make_command("hours", None));
        assert_eq!(result.reply, "Mon-Sun 9am-10pm");
    }

    #[test]
    fn active_workflow_uses_derived_task_and_prompts_next_slot() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![
            analysis(
                "reservation_create",
                vec![
                    entity("person", "Jean Martin"),
                    entity("date", "June 12"),
                    entity("time", "7pm"),
                ],
            ),
            analysis("reservation_create", vec![entity("people_count", "4 people")]),
        ]));
        let use_case = make_use_case(analyzer.clone());

        let start = use_case.handle_message(make_command("book", None));
        let next = use_case.handle_message(make_command("for 4 people", Some(&start.session_id)));

        assert_eq!(start.reply, "For how many people?");
        assert_eq!(
            next.reply,
            "I have the reservation details. Do you confirm this reservation?"
        );
        assert_eq!(
            analyzer.recorded_tasks(),
            vec![None, Some("WF_RESERVATION_CREATE".to_string())]
        );
    }

    #[test]
    fn ready_for_confirmation_uses_choice_task() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![
            analysis(
                "reservation_create",
                vec![
                    entity("person", "Jean Martin"),
                    entity("date", "June 12"),
                    entity("time", "7pm"),
                    entity("people_count", "4 people"),
                ],
            ),
            analysis("affirmative", vec![]),
        ]));
        let use_case = make_use_case(analyzer.clone());

        let start = use_case.handle_message(make_command("book", None));
        let confirm = use_case.handle_message(make_command("yes", Some(&start.session_id)));

        assert_eq!(
            start.reply,
            "I have the reservation details. Do you confirm this reservation?"
        );
        assert_eq!(confirm.reply, "Your reservation request is confirmed.");
        assert_eq!(
            analyzer.recorded_tasks(),
            vec![None, Some("WF_CHOICE".to_string())]
        );
    }

    #[test]
    fn cancel_intent_cancels_active_workflow() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![
            analysis("reservation_create", vec![entity("person", "Jean Martin")]),
            analysis("cancel", vec![]),
        ]));
        let use_case = make_use_case(analyzer);

        let start = use_case.handle_message(make_command("book", None));
        let cancel = use_case.handle_message(make_command("cancel", Some(&start.session_id)));

        assert_eq!(cancel.reply, "Okay, I cancelled the current workflow.");
    }
}
