use std::str::FromStr;

use super::conversation_command::{HandleConversationCommand, HandleConversationResult};
use super::conversation_processor::ConversationProcessor;
use super::port::inbound::conversation_usecase::HandleConversationUseCase;
use super::port::outbound::conversation_repository_port::ConversationRepositoryPort;
use super::port::outbound::language_detector_port::LanguageDetectorPort;
use super::port::outbound::nlp_engine_gateway_port::NlpEngineGatewayPort;
use crate::core::conversation::domain::conversation::Conversation;
use crate::core::conversation::domain::conversation_id::ConversationId;
use crate::core::conversation::domain::domain_type::DomainType;

pub struct HandleConversationService<N, R, L>
where
    N: NlpEngineGatewayPort,
    R: ConversationRepositoryPort,
    L: LanguageDetectorPort,
{
    domain: DomainType,
    nlu_engine_gateway: N,
    conversation_repository: R,
    language_detector: L,
    processor: ConversationProcessor,
}

impl<N, R, L> HandleConversationUseCase for HandleConversationService<N, R, L>
where
    N: NlpEngineGatewayPort,
    R: ConversationRepositoryPort,
    L: LanguageDetectorPort,
{
    fn handle_message(&self, command: HandleConversationCommand) -> HandleConversationResult {
        let (conversation, session_id) =
            self.load_or_create_conversation(command.session_id.as_deref(), &command.message);
        let analysis = self.nlu_engine_gateway.analyze(
            &command.message,
            &conversation.lang,
            conversation.domain,
            conversation.detect_task(),
        );
        self.log_nlu_analysis(&session_id, &command.message, &conversation, &analysis);

        let process_result = self
            .processor
            .process(conversation, &command.message, analysis);

        let _ = self
            .conversation_repository
            .save(&process_result.updated_conversation);

        HandleConversationResult {
            session_id,
            reply: process_result.reply,
        }
    }
}

impl<N, R, L> HandleConversationService<N, R, L>
where
    N: NlpEngineGatewayPort,
    R: ConversationRepositoryPort,
    L: LanguageDetectorPort,
{
    pub fn new(
        domain: DomainType,
        processor: ConversationProcessor,
        nlu_engine_gateway: N,
        conversation_repository: R,
        language_detector: L,
    ) -> Self {
        Self {
            domain,
            nlu_engine_gateway,
            conversation_repository,
            language_detector,
            processor,
        }
    }

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

            let mut conversation = Conversation::with_id(conversation_id, self.domain);
            conversation.lang = self.language_detector.detect(message);
            return (conversation, conversation_id.to_string());
        }

        let mut conversation = Conversation::new(self.domain);
        conversation.lang = self.language_detector.detect(message);
        let session_id = conversation.id.to_string();
        (conversation, session_id)
    }

    fn log_nlu_analysis(
        &self,
        session_id: &str,
        message: &str,
        conversation: &Conversation,
        analysis: &crate::core::nlu_engine::domain::analysis::NluAnalysis,
    ) {
        if !debug_nlu_logging_enabled() {
            return;
        }

        let task = conversation
            .detect_task()
            .map(|value| value.as_tag().to_string())
            .unwrap_or_else(|| "-".to_string());
        let intents = if analysis.intents.is_empty() {
            "-".to_string()
        } else {
            analysis
                .intents
                .iter()
                .map(|intent| format!("{}:{:.3}", intent.name, intent.confidence))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let entities = if analysis.entities.is_empty() {
            "-".to_string()
        } else {
            analysis
                .entities
                .iter()
                .map(|entity| format!("{:?}={}", entity.entity_type, entity.value))
                .collect::<Vec<_>>()
                .join(", ")
        };

        eprintln!(
            "[nlu] session={session_id} domain={} lang={} task={} text={message:?} intent={}:{:.3} candidates=[{}] entities=[{}]",
            conversation.domain.as_str(),
            conversation.lang,
            task,
            analysis.intent.name,
            analysis.intent.confidence,
            intents,
            entities,
        );
    }
}

fn debug_nlu_logging_enabled() -> bool {
    matches!(
        std::env::var("COREBOT_DEBUG_NLU"),
        Ok(value) if matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, RwLock};

    use super::*;
    use crate::core::conversation::application::intent_handler::IntentHandlerRegistry;
    use crate::core::conversation::application::port::outbound::conversation_repository_port::{
        ConversationRepositoryPort, RepositoryError,
    };
    use crate::core::conversation::application::port::outbound::language_detector_port::LanguageDetectorPort;
    use crate::core::conversation::application::port::outbound::nlp_engine_gateway_port::NlpEngineGatewayPort;
    use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
    use crate::core::conversation::application::port::outbound::restaurant_queries::{
        EventQuery, FacilityQuery, LocationQuery, MenuDietaryQuery, MenuItemDetailsQuery,
        MenuQuery, PaymentMethodQuery, PriceQuery, ReservationCreateQuery, ReservationLookupQuery,
    };
    use crate::core::conversation::application::port::outbound::restaurant_reservation_port::RestaurantReservationPort;
    use crate::core::conversation::application::restaurant_handler_registry_factory::{
        RestaurantConversationDependencies, RestaurantHandlerRegistryFactory,
    };
    use crate::core::conversation::domain::date_resolver::{DateResolveError, DateResolver};
    use crate::core::conversation::domain::model::intent::NluTask;
    use crate::core::conversation::domain::slot::EntityType;
    use crate::core::nlu_engine::domain::analysis::{
        NerTokenLabel, NluAnalysis, NluEntity, NluIntent, NluIntentCandidate,
    };

    #[derive(Clone)]
    struct StubInformationPort {
        calls: Arc<Mutex<u32>>,
    }

    impl StubInformationPort {
        fn new() -> Self {
            Self {
                calls: Arc::new(Mutex::new(0)),
            }
        }

        fn calls(&self) -> u32 {
            *self.calls.lock().unwrap()
        }
    }

    impl RestaurantInformationPort for StubInformationPort {
        fn get_opening_hours(&self) -> String {
            *self.calls.lock().unwrap() += 1;
            "Mon-Sun 9am-10pm".to_string()
        }

        fn find_menu(&self, _: MenuQuery) -> String {
            "full_menu:".to_string()
        }

        fn find_menu_dietary(&self, _: MenuDietaryQuery) -> String {
            "dietary_no_filter:".to_string()
        }

        fn find_menu_item_details(&self, _: MenuItemDetailsQuery) -> String {
            "details_no_filter:".to_string()
        }

        fn find_location(&self, _: LocationQuery) -> String {
            "address:".to_string()
        }

        fn get_contact(&self) -> String {
            "contact:+33123456789|test@example.com".to_string()
        }

        fn find_payment_methods(&self, _: PaymentMethodQuery) -> String {
            "all_methods:cash".to_string()
        }

        fn find_price(&self, _: PriceQuery) -> String {
            "price_general:".to_string()
        }

        fn get_takeaway_info(&self) -> String {
            "takeaway:yes|Yes".to_string()
        }

        fn find_event_info(&self, _: EventQuery) -> String {
            "event_info:Yes".to_string()
        }

        fn find_facility_info(&self, _: FacilityQuery) -> String {
            "all_facilities:wifi".to_string()
        }

        fn get_accessibility_info(&self) -> String {
            "accessibility:yes|Yes".to_string()
        }

        fn get_entertainment_info(&self) -> String {
            "entertainment:yes|Live music".to_string()
        }
    }

    #[derive(Clone)]
    struct StubReservationPort;

    impl RestaurantReservationPort for StubReservationPort {
        fn create_reservation(&self, _: ReservationCreateQuery) -> String {
            "created:REST-NEW123".to_string()
        }

        fn check_reservation(&self, _: ReservationLookupQuery) -> String {
            "no_reference_or_name:".to_string()
        }
    }

    #[derive(Clone)]
    struct StubLanguageDetector {
        lang: Arc<Mutex<String>>,
        calls: Arc<Mutex<u32>>,
    }

    impl StubLanguageDetector {
        fn new(lang: &str) -> Self {
            Self {
                lang: Arc::new(Mutex::new(lang.to_string())),
                calls: Arc::new(Mutex::new(0)),
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

    #[derive(Clone)]
    struct StubConversationRepository {
        store: Arc<RwLock<HashMap<ConversationId, Conversation>>>,
        save_calls: Arc<Mutex<u32>>,
    }

    impl StubConversationRepository {
        fn new() -> Self {
            Self {
                store: Arc::new(RwLock::new(HashMap::new())),
                save_calls: Arc::new(Mutex::new(0)),
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
                .insert(conversation.id, conversation.clone());
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

    #[derive(Clone)]
    struct StubNlpAnalyzer {
        responses: Arc<Mutex<Vec<NluAnalysis>>>,
        tasks: Arc<Mutex<Vec<Option<String>>>>,
        langs: Arc<Mutex<Vec<String>>>,
        domains: Arc<Mutex<Vec<String>>>,
    }

    impl StubNlpAnalyzer {
        fn new(responses: Vec<NluAnalysis>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses.into_iter().rev().collect())),
                tasks: Arc::new(Mutex::new(vec![])),
                langs: Arc::new(Mutex::new(vec![])),
                domains: Arc::new(Mutex::new(vec![])),
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
                .push(task.map(|current| current.as_tag().to_string()));
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

    struct AlwaysOk;

    impl DateResolver for AlwaysOk {
        fn resolve(
            &self,
            _: &str,
            today: chrono::NaiveDate,
        ) -> Result<chrono::NaiveDate, DateResolveError> {
            Ok(today + chrono::Duration::days(1))
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

    fn entity(entity_type: EntityType, value: &str) -> NluEntity {
        NluEntity {
            entity_type,
            value: value.to_string(),
            raw_value: value.to_string(),
            start: 0,
            end: value.len(),
            confidence: 1.0,
        }
    }

    type TestUseCase = HandleConversationService<
        StubNlpAnalyzer,
        StubConversationRepository,
        StubLanguageDetector,
    >;

    struct UseCaseParts {
        use_case: TestUseCase,
        repo: StubConversationRepository,
        detector: StubLanguageDetector,
        information_port: StubInformationPort,
    }

    fn make_use_case(analyzer: StubNlpAnalyzer) -> UseCaseParts {
        make_use_case_for_domain(DomainType::Restaurant, analyzer, "en")
    }

    fn make_use_case_for_domain(
        domain: DomainType,
        analyzer: StubNlpAnalyzer,
        lang: &str,
    ) -> UseCaseParts {
        let repo = StubConversationRepository::new();
        let detector = StubLanguageDetector::new(lang);
        let information_port = StubInformationPort::new();
        let reservation_port = StubReservationPort;
        let restaurant_registry =
            RestaurantHandlerRegistryFactory::build(RestaurantConversationDependencies {
                information_port: Arc::new(information_port.clone()),
                reservation_port: Arc::new(reservation_port),
                date_resolver: Arc::new(AlwaysOk),
            });
        let processor =
            ConversationProcessor::new(restaurant_registry, IntentHandlerRegistry::new(vec![]));
        let use_case = HandleConversationService::new(
            domain,
            processor,
            analyzer,
            repo.clone(),
            detector.clone(),
        );
        UseCaseParts {
            use_case,
            repo,
            detector,
            information_port,
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
        let analyzer = StubNlpAnalyzer::new(vec![analysis("greeting", vec![])]);
        let parts = make_use_case(analyzer);

        let result = parts
            .use_case
            .handle_message(make_command("hello", Some(&session_id)));

        assert_eq!(result.session_id, session_id);
    }

    #[test]
    fn handle_message_generates_session_id_when_none() {
        let analyzer = StubNlpAnalyzer::new(vec![analysis("greeting", vec![])]);

        let result = make_use_case(analyzer)
            .use_case
            .handle_message(make_command("hello", None));

        assert!(!result.session_id.is_empty());
    }

    #[test]
    fn dynamic_domain_response_calls_restaurant_gateway() {
        let analyzer = StubNlpAnalyzer::new(vec![analysis("ask_opening_hours", vec![])]);
        let parts = make_use_case(analyzer);

        let result = parts.use_case.handle_message(make_command("hours", None));

        assert_eq!(result.reply, "Mon-Sun 9am-10pm");
        assert_eq!(parts.information_port.calls(), 1);
    }

    #[test]
    fn active_workflow_uses_derived_task_and_prompts_next_slot() {
        let analyzer = StubNlpAnalyzer::new(vec![
            analysis(
                "reservation_create",
                vec![
                    entity(EntityType::Person, "Jean Martin"),
                    entity(EntityType::Date, "June 12"),
                    entity(EntityType::Time, "7pm"),
                ],
            ),
            analysis(
                "reservation_create",
                vec![entity(EntityType::PeopleCount, "4 people")],
            ),
        ]);
        let parts = make_use_case(analyzer.clone());

        let start = parts.use_case.handle_message(make_command("book", None));
        let next = parts
            .use_case
            .handle_message(make_command("for 4 people", Some(&start.session_id)));

        assert_eq!(start.reply, "For how many people?");
        assert_eq!(
            next.reply,
            "I have the reservation details: Jean Martin, June 12 at 7pm, for 4 people. Do you confirm this reservation?"
        );
        let recorded = analyzer.recorded_tasks();
        assert_eq!(recorded.len(), 2);
        assert!(recorded[0].is_none());
        assert_eq!(recorded[1], Some("WF_RESERVATION_CREATE".to_string()));
    }

    #[test]
    fn ready_for_confirmation_uses_choice_task() {
        let analyzer = StubNlpAnalyzer::new(vec![
            analysis(
                "reservation_create",
                vec![
                    entity(EntityType::Person, "Jean Martin"),
                    entity(EntityType::Date, "June 12"),
                    entity(EntityType::Time, "7pm"),
                    entity(EntityType::PeopleCount, "4 people"),
                ],
            ),
            analysis("affirmative", vec![]),
        ]);
        let parts = make_use_case(analyzer.clone());

        let start = parts.use_case.handle_message(make_command("book", None));
        let confirm = parts
            .use_case
            .handle_message(make_command("yes", Some(&start.session_id)));

        assert_eq!(
            start.reply,
            "I have the reservation details: Jean Martin, June 12 at 7pm, for 4 people. Do you confirm this reservation?"
        );
        assert_eq!(
            confirm.reply,
            "Your reservation is confirmed for Jean Martin, June 12 at 7pm, for 4 people. Your reference is REST-NEW123."
        );
        let recorded = analyzer.recorded_tasks();
        assert_eq!(recorded.len(), 2);
        assert!(recorded[0].is_none());
        assert_eq!(recorded[1], Some("WF_CHOICE".to_string()));
    }

    #[test]
    fn cancel_intent_cancels_active_workflow() {
        let analyzer = StubNlpAnalyzer::new(vec![
            analysis(
                "reservation_create",
                vec![entity(EntityType::Person, "Jean Martin")],
            ),
            analysis("cancel", vec![]),
        ]);
        let parts = make_use_case(analyzer);

        let start = parts.use_case.handle_message(make_command("book", None));
        let cancel = parts
            .use_case
            .handle_message(make_command("cancel", Some(&start.session_id)));

        assert_eq!(cancel.reply, "Okay, I cancelled the current workflow.");
    }

    #[test]
    fn renderer_returns_indonesian_localized_prompt() {
        let analyzer = StubNlpAnalyzer::new(vec![analysis(
            "reservation_create",
            vec![
                entity(EntityType::Person, "Budi Santoso"),
                entity(EntityType::Date, "besok"),
                entity(EntityType::Time, "jam 7 malam"),
            ],
        )]);
        let parts = make_use_case_for_domain(DomainType::Restaurant, analyzer.clone(), "id");

        let result = parts
            .use_case
            .handle_message(make_command("Halo, saya mau pesan meja", None));

        assert_eq!(result.reply, "Untuk berapa orang?");
        assert_eq!(analyzer.recorded_langs(), vec!["id".to_string()]);
    }

    #[test]
    fn injected_domain_is_forwarded_to_nlu() {
        let analyzer = StubNlpAnalyzer::new(vec![analysis("greeting", vec![])]);
        let parts = make_use_case_for_domain(DomainType::Hotel, analyzer.clone(), "en");

        let result = parts.use_case.handle_message(make_command("hello", None));

        assert_eq!(result.reply, "Detected intent: greeting");
        assert_eq!(analyzer.recorded_domains(), vec!["hotel".to_string()]);
    }

    #[test]
    fn language_detector_is_used_only_for_new_sessions() {
        let analyzer = StubNlpAnalyzer::new(vec![
            analysis("greeting", vec![]),
            analysis("thanks", vec![]),
        ]);
        let parts = make_use_case_for_domain(DomainType::Restaurant, analyzer.clone(), "id");

        let first = parts.use_case.handle_message(make_command("halo", None));
        let _second = parts
            .use_case
            .handle_message(make_command("thanks", Some(&first.session_id)));

        assert_eq!(parts.detector.calls(), 1);
    }

    #[test]
    fn existing_session_preserves_stored_language() {
        let analyzer = StubNlpAnalyzer::new(vec![
            analysis("greeting", vec![]),
            analysis("thanks", vec![]),
        ]);
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
        let analyzer = StubNlpAnalyzer::new(vec![analysis("reservation_create", vec![])]);
        let parts = make_use_case(analyzer);

        let _ = parts.use_case.handle_message(make_command("book", None));

        assert_eq!(parts.repo.save_calls(), 1);
    }
}
