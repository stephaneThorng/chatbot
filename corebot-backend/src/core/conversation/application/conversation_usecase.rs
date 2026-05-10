use std::str::FromStr;
use std::sync::Arc;

use super::conversation_command::{HandleConversationCommand, HandleConversationResult};
use super::conversation_processor::ConversationProcessor;
use super::port::inbound::conversation_trait::HandleConversationPort;
use super::port::outbound::conversation_repository::ConversationRepositoryPort;
use super::port::outbound::domain_gateway_trait::DomainGatewayPort;
use super::port::outbound::language_detector_trait::LanguageDetectorPort;
use super::port::outbound::nlp_analyzer_trait::NlpEngineGatewayPort;
use crate::core::conversation::domain::conversation::Conversation;
use crate::core::conversation::domain::conversation_id::ConversationId;
use crate::core::conversation::domain::domain_type::DomainType;

/// Use case that handles one user message in a conversation session.
///
/// It stays intentionally thin: session lifecycle, NLU call, conversation
/// processing delegation, persistence, and response assembly.
pub struct HandleConversationUseCase {
    domain: DomainType,
    nlu_engine_gateway: Arc<dyn NlpEngineGatewayPort>,
    conversation_repository: Arc<dyn ConversationRepositoryPort>,
    language_detector: Arc<dyn LanguageDetectorPort>,
    processor: ConversationProcessor,
}

impl HandleConversationPort for HandleConversationUseCase {
    fn handle_message(&self, command: HandleConversationCommand) -> HandleConversationResult {
        let (conversation, session_id) =
            self.load_or_create_conversation(command.session_id.as_deref(), &command.message);
        let analysis = self.nlu_engine_gateway.analyze(
            &command.message,
            &conversation.lang,
            conversation.domain,
            self.processor.detect_task(&conversation),
        );

        let process_result = self
            .processor
            .process(&conversation, &command.message, analysis);

        let _ = self
            .conversation_repository
            .save(&process_result.updated_conversation);

        HandleConversationResult {
            session_id,
            reply: process_result.reply,
        }
    }
}

impl HandleConversationUseCase {
    pub fn new(
        domain: DomainType,
        domain_gateway: Arc<dyn DomainGatewayPort>,
        nlu_engine_gateway: Arc<dyn NlpEngineGatewayPort>,
        conversation_repository: Arc<dyn ConversationRepositoryPort>,
        language_detector: Arc<dyn LanguageDetectorPort>,
    ) -> Self {
        Self {
            domain,
            nlu_engine_gateway,
            conversation_repository,
            language_detector,
            processor: ConversationProcessor::new(domain_gateway),
        }
    }

    /// Loads a stored conversation or creates a new one for the configured domain.
    ///
    /// Language detection runs only for new sessions so an existing conversation
    /// keeps the language selected at its first turn.
    fn load_or_create_conversation(
        &self,
        session_id: Option<&str>,
        message: &str,
    ) -> (Conversation, String) {
        let parsed_id = session_id.and_then(|id| ConversationId::from_str(id).ok());
        if let Some(conversation_id) = parsed_id {
            if let Ok(Some(conversation)) = self.conversation_repository.load(&conversation_id) {
                return (conversation, conversation_id.to_string());
            }

            let mut conversation = Conversation::with_id(conversation_id.clone(), self.domain);
            conversation.lang = self.language_detector.detect(message);
            return (conversation, conversation_id.to_string());
        }

        let mut conversation = Conversation::new(self.domain);
        conversation.lang = self.language_detector.detect(message);
        let session_id = conversation.id.to_string();
        (conversation, session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;

    use crate::core::conversation::application::port::outbound::conversation_repository::RepositoryError;
    use crate::core::conversation::domain::intent::NluTask;
    use crate::core::nlu_engine::domain::analysis::{
        NerTokenLabel, NluAnalysis, NluEntity, NluIntent, NluIntentCandidate,
    };

    struct StubDomainGateway {
        calls: std::sync::Mutex<u32>,
    }

    impl StubDomainGateway {
        fn new() -> Self {
            Self {
                calls: std::sync::Mutex::new(0),
            }
        }

        fn calls(&self) -> u32 {
            *self.calls.lock().unwrap()
        }
    }

    impl DomainGatewayPort for StubDomainGateway {
        fn get_opening_hours(&self) -> String {
            *self.calls.lock().unwrap() += 1;
            "Mon-Sun 9am-10pm".to_string()
        }
    }

    struct StubLanguageDetector {
        lang: std::sync::Mutex<String>,
        calls: std::sync::Mutex<u32>,
    }

    impl StubLanguageDetector {
        fn new(lang: &str) -> Self {
            Self {
                lang: std::sync::Mutex::new(lang.to_string()),
                calls: std::sync::Mutex::new(0),
            }
        }

        fn calls(&self) -> u32 {
            *self.calls.lock().unwrap()
        }
    }

    impl LanguageDetectorPort for StubLanguageDetector {
        fn detect(&self, _text: &str) -> String {
            *self.calls.lock().unwrap() += 1;
            self.lang.lock().unwrap().clone()
        }
    }

    struct StubConversationRepository {
        store: RwLock<HashMap<ConversationId, Conversation>>,
        save_calls: std::sync::Mutex<u32>,
    }

    impl StubConversationRepository {
        fn new() -> Self {
            Self {
                store: RwLock::new(HashMap::new()),
                save_calls: std::sync::Mutex::new(0),
            }
        }

        fn save_calls(&self) -> u32 {
            *self.save_calls.lock().unwrap()
        }
    }

    impl ConversationRepositoryPort for StubConversationRepository {
        fn save(&self, conversation: &Conversation) -> Result<(), RepositoryError> {
            *self.save_calls.lock().unwrap() += 1;
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
        langs: std::sync::Mutex<Vec<String>>,
        domains: std::sync::Mutex<Vec<String>>,
    }

    impl StubNlpAnalyzer {
        fn new(responses: Vec<NluAnalysis>) -> Self {
            Self {
                responses: std::sync::Mutex::new(responses.into_iter().rev().collect()),
                tasks: std::sync::Mutex::new(vec![]),
                langs: std::sync::Mutex::new(vec![]),
                domains: std::sync::Mutex::new(vec![]),
            }
        }

        fn recorded_tasks(&self) -> Vec<Option<String>> {
            self.tasks.lock().unwrap().clone()
        }

        fn recorded_langs(&self) -> Vec<String> {
            self.langs.lock().unwrap().clone()
        }

        fn recorded_domains(&self) -> Vec<String> {
            self.domains.lock().unwrap().clone()
        }
    }

    impl NlpEngineGatewayPort for StubNlpAnalyzer {
        fn analyze(
            &self,
            text: &str,
            lang: &str,
            domain: DomainType,
            task: Option<NluTask>,
        ) -> NluAnalysis {
            let _ = text;
            self.tasks
                .lock()
                .unwrap()
                .push(task.map(|t| t.as_tag().to_string()));
            self.langs.lock().unwrap().push(lang.to_string());
            self.domains
                .lock()
                .unwrap()
                .push(domain.as_str().to_string());
            self.responses
                .lock()
                .unwrap()
                .pop()
                .expect("missing stub NLU response")
        }
    }

    fn analysis(intent_name: &'static str, entities: Vec<NluEntity>) -> NluAnalysis {
        NluAnalysis {
            processed_text: String::new(),
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

    struct UseCaseParts {
        use_case: HandleConversationUseCase,
        repo: Arc<StubConversationRepository>,
        detector: Arc<StubLanguageDetector>,
        domain_gateway: Arc<StubDomainGateway>,
    }

    fn make_use_case(analyzer: Arc<StubNlpAnalyzer>) -> UseCaseParts {
        make_use_case_for_domain(DomainType::Restaurant, analyzer, "en")
    }

    fn make_use_case_for_domain(
        domain: DomainType,
        analyzer: Arc<StubNlpAnalyzer>,
        lang: &str,
    ) -> UseCaseParts {
        let repo = Arc::new(StubConversationRepository::new());
        let detector = Arc::new(StubLanguageDetector::new(lang));
        let domain_gateway = Arc::new(StubDomainGateway::new());
        let use_case = HandleConversationUseCase::new(
            domain,
            domain_gateway.clone(),
            analyzer,
            repo.clone(),
            detector.clone(),
        );
        UseCaseParts {
            use_case,
            repo,
            detector,
            domain_gateway,
        }
    }

    fn make_command(message: &str, session_id: Option<&str>) -> HandleConversationCommand {
        HandleConversationCommand {
            message: message.to_string(),
            session_id: session_id.map(str::to_string),
        }
    }

    #[test]
    fn handle_message_reuses_provided_session_id() {
        let session_id = ConversationId::new().to_string();
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![analysis("greeting", vec![])]));
        let parts = make_use_case(analyzer);

        let result = parts
            .use_case
            .handle_message(make_command("hello", Some(&session_id)));

        assert_eq!(result.session_id, session_id);
    }

    #[test]
    fn handle_message_generates_session_id_when_none() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![analysis("greeting", vec![])]));

        let result = make_use_case(analyzer)
            .use_case
            .handle_message(make_command("hello", None));

        assert!(!result.session_id.is_empty());
    }

    #[test]
    fn dynamic_domain_response_calls_restaurant_gateway() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![analysis(
            "ask_opening_hours",
            vec![],
        )]));
        let parts = make_use_case(analyzer);

        let result = parts.use_case.handle_message(make_command("hours", None));

        assert_eq!(result.reply, "Mon-Sun 9am-10pm");
        assert_eq!(parts.domain_gateway.calls(), 1);
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
            analysis(
                "reservation_create",
                vec![entity("people_count", "4 people")],
            ),
        ]));
        let parts = make_use_case(analyzer.clone());

        let start = parts.use_case.handle_message(make_command("book", None));
        let next = parts
            .use_case
            .handle_message(make_command("for 4 people", Some(&start.session_id)));

        assert_eq!(start.reply, "For how many people?");
        assert_eq!(
            next.reply,
            "I have the reservation details. Do you confirm this reservation?"
        );
        let recorded = analyzer.recorded_tasks();
        assert_eq!(recorded.len(), 2);
        assert!(recorded[0].is_none());
        assert_eq!(recorded[1], Some("WF_RESERVATION_CREATE".to_string()));
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
        let parts = make_use_case(analyzer.clone());

        let start = parts.use_case.handle_message(make_command("book", None));
        let confirm = parts
            .use_case
            .handle_message(make_command("yes", Some(&start.session_id)));

        assert_eq!(
            start.reply,
            "I have the reservation details. Do you confirm this reservation?"
        );
        assert_eq!(confirm.reply, "Your reservation request is confirmed.");
        let recorded = analyzer.recorded_tasks();
        assert_eq!(recorded.len(), 2);
        assert!(recorded[0].is_none());
        assert_eq!(recorded[1], Some("WF_CHOICE".to_string()));
    }

    #[test]
    fn cancel_intent_cancels_active_workflow() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![
            analysis("reservation_create", vec![entity("person", "Jean Martin")]),
            analysis("cancel", vec![]),
        ]));
        let parts = make_use_case(analyzer);

        let start = parts.use_case.handle_message(make_command("book", None));
        let cancel = parts
            .use_case
            .handle_message(make_command("cancel", Some(&start.session_id)));

        assert_eq!(cancel.reply, "Okay, I cancelled the current workflow.");
    }

    #[test]
    fn renderer_returns_indonesian_localized_prompt() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![analysis(
            "reservation_create",
            vec![
                entity("person", "Budi Santoso"),
                entity("date", "besok"),
                entity("time", "jam 7 malam"),
            ],
        )]));
        let parts = make_use_case_for_domain(DomainType::Restaurant, analyzer.clone(), "id");

        let result = parts
            .use_case
            .handle_message(make_command("Halo, saya mau pesan meja", None));

        assert_eq!(result.reply, "Untuk berapa orang?");
        assert_eq!(analyzer.recorded_langs(), vec!["id".to_string()]);
    }

    #[test]
    fn injected_domain_is_forwarded_to_nlu() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![analysis("greeting", vec![])]));
        let parts = make_use_case_for_domain(DomainType::Hotel, analyzer.clone(), "en");

        let result = parts.use_case.handle_message(make_command("hello", None));

        assert_eq!(result.reply, "Detected intent: greeting");
        assert_eq!(analyzer.recorded_domains(), vec!["hotel".to_string()]);
    }

    #[test]
    fn language_detector_is_used_only_for_new_sessions() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![
            analysis("greeting", vec![]),
            analysis("thanks", vec![]),
        ]));
        let parts = make_use_case_for_domain(DomainType::Restaurant, analyzer.clone(), "id");

        let first = parts.use_case.handle_message(make_command("halo", None));
        let _second = parts
            .use_case
            .handle_message(make_command("thanks", Some(&first.session_id)));

        assert_eq!(parts.detector.calls(), 1);
    }

    #[test]
    fn existing_session_preserves_stored_language() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![
            analysis("greeting", vec![]),
            analysis("thanks", vec![]),
        ]));
        let parts = make_use_case_for_domain(DomainType::Restaurant, analyzer.clone(), "id");

        let first = parts.use_case.handle_message(make_command("halo", None));
        *parts.detector.lang.lock().unwrap() = "en".to_string();
        let _second = parts
            .use_case
            .handle_message(make_command("thanks", Some(&first.session_id)));

        assert_eq!(
            analyzer.recorded_langs(),
            vec!["id".to_string(), "id".to_string()]
        );
    }

    #[test]
    fn repository_save_is_called_after_transition() {
        let analyzer = Arc::new(StubNlpAnalyzer::new(vec![analysis(
            "reservation_create",
            vec![],
        )]));
        let parts = make_use_case(analyzer);

        let _ = parts.use_case.handle_message(make_command("book", None));

        assert_eq!(parts.repo.save_calls(), 1);
    }
}
