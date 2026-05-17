use std::str::FromStr;

use super::conversation_processor::ConversationProcessor;
use super::dto::conversation_command::{HandleConversationCommand, HandleConversationResult};
use super::intent_handler::intent_handler::IntentHandlerRegistry;
use super::intent_handler::restaurant_handler_registry_factory::{
    RestaurantConversationDependencies, RestaurantHandlerRegistryFactory,
};
use super::port::inbound::conversation_usecase::HandleConversationUseCase;
use super::port::outbound::conversation_repository_port::ConversationRepositoryPort;
use super::port::outbound::language_detector_port::LanguageDetectorPort;
use super::port::outbound::nlp_engine_gateway_port::NlpEngineGatewayPort;
use super::port::outbound::restaurant::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use super::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use super::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use super::port::outbound::restaurant::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
use super::service::restaurant::{
    ConversationRestaurantMenuService, ConversationRestaurantReservationService,
};
use crate::core::conversation::application::dto::nlu_analysis_result::NluAnalysisResult;
use crate::core::conversation::domain::conversation::Conversation;
use crate::core::conversation::domain::conversation_id::ConversationId;
use crate::core::conversation::domain::domain_type::DomainType;
use rust_i18n::t;
use uuid::Uuid;

pub struct HandleConversationService<N, CR, L, B, M, RR, A>
where
    N: NlpEngineGatewayPort,
    CR: ConversationRepositoryPort,
    L: LanguageDetectorPort,
    B: RestaurantBusinessInfoRepositoryPort,
    M: RestaurantMenuRepositoryPort,
    RR: RestaurantReservationRepositoryPort,
    A: RestaurantAvailabilityRepositoryPort,
{
    domain: DomainType,
    nlu_engine_gateway: N,
    conversation_repository: CR,
    language_detector: L,
    processor: ConversationProcessor,
    restaurant_business_id: Uuid,
    restaurant_business_info_repository: B,
    restaurant_menu_service: ConversationRestaurantMenuService<M>,
    restaurant_reservation_service: ConversationRestaurantReservationService<RR, A>,
}

#[async_trait::async_trait]
impl<N, CR, L, B, M, RR, A> HandleConversationUseCase
    for HandleConversationService<N, CR, L, B, M, RR, A>
where
    N: NlpEngineGatewayPort + Send + Sync,
    CR: ConversationRepositoryPort + Send + Sync,
    L: LanguageDetectorPort + Send + Sync,
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
    M: RestaurantMenuRepositoryPort + Send + Sync,
    RR: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
{
    async fn handle_message(&self, command: HandleConversationCommand) -> HandleConversationResult {
        let (conversation, session_id, is_new_session) =
            self.load_or_create_conversation(command.session_id.as_deref(), &command.message);
        let analysis = self.nlu_engine_gateway.analyze(
            &command.message,
            &conversation.lang,
            conversation.domain,
            conversation.detect_task(),
            conversation.detect_slot_hint(),
        );
        self.log_nlu_analysis(&session_id, &command.message, &conversation, &analysis);

        let restaurant_registry = match conversation.domain {
            DomainType::Restaurant => {
                RestaurantHandlerRegistryFactory::build(RestaurantConversationDependencies {
                    business_info_repository: &self.restaurant_business_info_repository,
                    menu_service: &self.restaurant_menu_service,
                    reservation_service: &self.restaurant_reservation_service,
                })
            }
            DomainType::Hotel => IntentHandlerRegistry::new(vec![]),
        };
        let welcome_lang = conversation.lang.clone();
        let process_result = self
            .processor
            .process_async(
                &restaurant_registry,
                conversation,
                &command.message,
                analysis,
            )
            .await;

        let _ = self
            .conversation_repository
            .save(&process_result.updated_conversation);

        let mut reply = process_result.reply;
        if is_new_session {
            reply.insert(
                0,
                t!("system.welcome", locale = welcome_lang.as_str()).to_string(),
            );
        }

        HandleConversationResult {
            session_id,
            reply,
        }
    }
}

impl<N, CR, L, B, M, RR, A> HandleConversationService<N, CR, L, B, M, RR, A>
where
    N: NlpEngineGatewayPort,
    CR: ConversationRepositoryPort,
    L: LanguageDetectorPort,
    B: RestaurantBusinessInfoRepositoryPort,
    M: RestaurantMenuRepositoryPort,
    RR: RestaurantReservationRepositoryPort,
    A: RestaurantAvailabilityRepositoryPort,
{
    pub fn new(
        domain: DomainType,
        processor: ConversationProcessor,
        nlu_engine_gateway: N,
        conversation_repository: CR,
        language_detector: L,
        restaurant_business_id: Uuid,
        restaurant_business_info_repository: B,
        restaurant_menu_service: ConversationRestaurantMenuService<M>,
        restaurant_reservation_service: ConversationRestaurantReservationService<RR, A>,
    ) -> Self {
        Self {
            domain,
            nlu_engine_gateway,
            conversation_repository,
            language_detector,
            processor,
            restaurant_business_id,
            restaurant_business_info_repository,
            restaurant_menu_service,
            restaurant_reservation_service,
        }
    }

    #[cfg(test)]
    pub fn handle_message(&self, command: HandleConversationCommand) -> HandleConversationResult
    where
        N: Send + Sync,
        CR: Send + Sync,
        L: Send + Sync,
        B: Send + Sync,
        M: Send + Sync,
        RR: Send + Sync,
        A: Send + Sync,
    {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime should be created")
            .block_on(<Self as HandleConversationUseCase>::handle_message(
                self, command,
            ))
    }

    fn load_or_create_conversation(
        &self,
        session_id: Option<&str>,
        message: &str,
    ) -> (Conversation, String, bool) {
        let parsed_id = session_id.and_then(|id| ConversationId::from_str(id).ok());
        if let Some(conversation_id) = parsed_id {
            if let Ok(Some(conversation)) = self.conversation_repository.load(&conversation_id) {
                return (conversation, conversation_id.to_string(), false);
            }

            let mut conversation = Conversation::with_id_for_business(
                conversation_id,
                self.domain,
                self.restaurant_business_id,
            );
            conversation.lang = self.language_detector.detect(message);
            return (conversation, conversation_id.to_string(), true);
        }

        let mut conversation =
            Conversation::new_for_business(self.domain, self.restaurant_business_id);
        conversation.lang = self.language_detector.detect(message);
        let session_id = conversation.id.to_string();
        (conversation, session_id, true)
    }

    fn log_nlu_analysis(
        &self,
        session_id: &str,
        message: &str,
        conversation: &Conversation,
        analysis: &NluAnalysisResult,
    ) {
        if !debug_nlu_logging_enabled() {
            return;
        }

        let task = conversation
            .detect_task()
            .map(|value| value.as_tag().to_string())
            .unwrap_or_else(|| "-".to_string());
        let slot = conversation
            .detect_slot_hint()
            .map(|value| value.as_str().to_string())
            .unwrap_or_else(|| "-".to_string());
        let entities = if analysis.entities.is_empty() {
            "-".to_string()
        } else {
            analysis
                .entities
                .iter()
                .map(|entity| format!("{}={}", entity.entity_label, entity.value))
                .collect::<Vec<_>>()
                .join(", ")
        };

        println!(
            "[nlu] session={session_id} domain={} lang={} task={} slot={} text={message:?} intent={}:{:.3} entities=[{}]",
            conversation.domain.as_str(),
            conversation.lang,
            task,
            slot,
            analysis.intent_name,
            analysis.intent_confidence,
            entities,
        );
    }
}

fn debug_nlu_logging_enabled() -> bool {
    std::env::var("COREBOT_DEBUG_NLU")
        .ok()
        .as_deref()
        .map(is_truthy_env_value)
        .unwrap_or(false)
}

fn is_truthy_env_value(value: &str) -> bool {
    let normalized = value.trim().trim_matches('\'').trim_matches('"');
    matches!(
        normalized.to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex, RwLock};

    use super::*;
    use crate::core::conversation::application::dto::nlu_analysis_result::{
        NluAnalysisResult, NluEntityResult, NluIntentCandidate,
    };
    use crate::core::conversation::application::port::outbound::conversation_repository_port::{
        ConversationRepositoryPort, RepositoryError,
    };
    use crate::core::conversation::application::port::outbound::language_detector_port::LanguageDetectorPort;
    use crate::core::conversation::application::port::outbound::nlp_engine_gateway_port::NlpEngineGatewayPort;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
    use crate::core::conversation::application::service::restaurant::{
        ConversationRestaurantMenuService, ConversationRestaurantReservationService,
    };
    use crate::core::conversation::domain::model::intent::NluTask;
    use crate::core::conversation::domain::restaurant::model::{
        AmountComparator, BusinessFact, BusinessLocation, ContactChannel, EventSpace, Facility,
        MenuItem, OpeningHours, PaymentMethod, Reservation, ReservationDraft,
        ReservationSettings, RestaurantRepositoryError, TableType,
    };
    use chrono::{NaiveDate, NaiveTime, Weekday};
    use uuid::Uuid;

    #[derive(Clone)]
    struct StubRestaurantRepository {
        opening_hours_calls: Arc<Mutex<u32>>,
        reservations: Arc<Mutex<Vec<Reservation>>>,
    }

    impl StubRestaurantRepository {
        fn new() -> Self {
            Self {
                opening_hours_calls: Arc::new(Mutex::new(0)),
                reservations: Arc::new(Mutex::new(vec![])),
            }
        }

        fn opening_hours_calls(&self) -> u32 {
            *self.opening_hours_calls.lock().unwrap()
        }
    }

    fn business_id() -> Uuid {
        Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()
    }

    fn opening_hours() -> Vec<OpeningHours> {
        [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ]
        .into_iter()
        .map(|day| OpeningHours {
            day_of_week: day,
            opens_at: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            closes_at: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            is_closed: false,
        })
        .collect()
    }

    #[async_trait::async_trait]
    impl RestaurantBusinessInfoRepositoryPort for StubRestaurantRepository {
        async fn opening_hours(
            &self,
            _: Uuid,
        ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
            *self.opening_hours_calls.lock().unwrap() += 1;
            Ok(opening_hours())
        }

        async fn location(
            &self,
            _: Uuid,
        ) -> Result<Option<BusinessLocation>, RestaurantRepositoryError> {
            Ok(Some(BusinessLocation {
                address_line: "12 Rue de la Paix".to_string(),
                nearby_description: Some("near Central Station".to_string()),
            }))
        }

        async fn contact_channels(
            &self,
            _: Uuid,
        ) -> Result<Vec<ContactChannel>, RestaurantRepositoryError> {
            Ok(vec![
                ContactChannel {
                    channel_type: "phone".to_string(),
                    value: "+33123456789".to_string(),
                },
                ContactChannel {
                    channel_type: "email".to_string(),
                    value: "test@example.com".to_string(),
                },
            ])
        }

        async fn payment_methods(
            &self,
            _: Uuid,
        ) -> Result<Vec<PaymentMethod>, RestaurantRepositoryError> {
            Ok(vec![PaymentMethod {
                method_code: "cash".to_string(),
            }])
        }

        async fn facilities(&self, _: Uuid) -> Result<Vec<Facility>, RestaurantRepositoryError> {
            Ok(vec![Facility {
                facility_code: "wifi".to_string(),
                label: "wifi".to_string(),
            }])
        }

        async fn facts(
            &self,
            _: Uuid,
            _: &str,
        ) -> Result<Vec<BusinessFact>, RestaurantRepositoryError> {
            Ok(vec![
                BusinessFact {
                    fact_type: "takeaway".to_string(),
                    title: None,
                    content: "stub".to_string(),
                    metadata: BTreeMap::new(),
                },
                BusinessFact {
                    fact_type: "accessibility".to_string(),
                    title: None,
                    content: "stub".to_string(),
                    metadata: BTreeMap::new(),
                },
                BusinessFact {
                    fact_type: "entertainment".to_string(),
                    title: None,
                    content: "stub".to_string(),
                    metadata: BTreeMap::new(),
                },
            ])
        }

        async fn event_spaces(
            &self,
            _: Uuid,
        ) -> Result<Vec<EventSpace>, RestaurantRepositoryError> {
            Ok(vec![EventSpace {
                name: "main room".to_string(),
                description: Some("stub".to_string()),
                contact: None,
            }])
        }
    }

    #[async_trait::async_trait]
    impl RestaurantMenuRepositoryPort for StubRestaurantRepository {
        async fn menu_items(
            &self,
            _: Uuid,
            _: &str,
        ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
            Ok(vec![])
        }

        async fn menu_items_by_price(
            &self,
            _: Uuid,
            _: &str,
            _: &AmountComparator,
        ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
            Ok(vec![])
        }
    }

    #[async_trait::async_trait]
    impl RestaurantReservationRepositoryPort for StubRestaurantRepository {
        async fn next_reference_index(&self, _: Uuid) -> Result<i64, RestaurantRepositoryError> {
            Ok(self.reservations.lock().unwrap().len() as i64 + 1)
        }

        async fn create_reservation(
            &self,
            _: Uuid,
            reservation: ReservationDraft,
        ) -> Result<Reservation, RestaurantRepositoryError> {
            let reservation = Reservation {
                reference: reservation.reference,
                name: reservation.name,
                date: reservation.date,
                time: reservation.time,
                people_count: reservation.people_count,
            };
            self.reservations.lock().unwrap().push(reservation.clone());
            Ok(reservation)
        }

        async fn find_by_reference(
            &self,
            _: Uuid,
            reference: &str,
        ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
            Ok(self
                .reservations
                .lock()
                .unwrap()
                .iter()
                .find(|reservation| reservation.reference.eq_ignore_ascii_case(reference))
                .cloned())
        }

        async fn find_by_name(
            &self,
            _: Uuid,
            name: &str,
        ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
            Ok(self
                .reservations
                .lock()
                .unwrap()
                .iter()
                .filter(|reservation| reservation.name.eq_ignore_ascii_case(name))
                .cloned()
                .collect())
        }

        async fn cancel_by_reference(
            &self,
            _: Uuid,
            reference: &str,
        ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
            let mut reservations = self.reservations.lock().unwrap();
            let Some(index) = reservations
                .iter()
                .position(|reservation| reservation.reference.eq_ignore_ascii_case(reference))
            else {
                return Ok(None);
            };
            Ok(Some(reservations.remove(index)))
        }
    }

    #[async_trait::async_trait]
    impl RestaurantAvailabilityRepositoryPort for StubRestaurantRepository {
        async fn reservation_settings(
            &self,
            _: Uuid,
        ) -> Result<ReservationSettings, RestaurantRepositoryError> {
            Ok(ReservationSettings {
                slot_minutes: 120,
                max_lookup_days: 7,
            })
        }

        async fn table_types(&self, _: Uuid) -> Result<Vec<TableType>, RestaurantRepositoryError> {
            Ok(vec![TableType {
                capacity: 6,
                count: 2,
            }])
        }

        async fn opening_hours(
            &self,
            _: Uuid,
        ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
            Ok(opening_hours())
        }

        async fn is_closed_at(
            &self,
            _: Uuid,
            _: NaiveDate,
            _: NaiveTime,
            _: u32,
        ) -> Result<bool, RestaurantRepositoryError> {
            Ok(false)
        }

        async fn reservations_near(
            &self,
            _: Uuid,
            _: NaiveDate,
        ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
            Ok(self.reservations.lock().unwrap().clone())
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
        responses: Arc<Mutex<Vec<NluAnalysisResult>>>,
        tasks: Arc<Mutex<Vec<Option<String>>>>,
        slots: Arc<Mutex<Vec<Option<String>>>>,
        langs: Arc<Mutex<Vec<String>>>,
        domains: Arc<Mutex<Vec<String>>>,
    }

    impl StubNlpAnalyzer {
        fn new(responses: Vec<NluAnalysisResult>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses.into_iter().rev().collect())),
                tasks: Arc::new(Mutex::new(vec![])),
                slots: Arc::new(Mutex::new(vec![])),
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

        fn recorded_slots(&self) -> Vec<Option<String>> {
            self.slots.lock().unwrap().clone()
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
            slot_hint: Option<crate::core::conversation::domain::model::slot::SlotName>,
        ) -> NluAnalysisResult {
            let _ = text;
            self.tasks
                .lock()
                .unwrap()
                .push(task.map(|current| current.as_tag().to_string()));
            self.slots
                .lock()
                .unwrap()
                .push(slot_hint.map(|current| current.as_str().to_string()));
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

    fn analysis(intent_name: &'static str, entities: Vec<NluEntityResult>) -> NluAnalysisResult {
        NluAnalysisResult {
            intent_name: intent_name.to_string(),
            intent_confidence: 1.0,
            intent_candidates: Vec::<NluIntentCandidate>::new(),
            entities,
        }
    }

    fn entity(entity_label: &'static str, value: &str) -> NluEntityResult {
        NluEntityResult {
            entity_label: entity_label.to_string(),
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
        StubRestaurantRepository,
        StubRestaurantRepository,
        StubRestaurantRepository,
        StubRestaurantRepository,
    >;

    struct UseCaseParts {
        use_case: TestUseCase,
        repo: StubConversationRepository,
        detector: StubLanguageDetector,
        restaurant_repository: StubRestaurantRepository,
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
        let restaurant_repository = StubRestaurantRepository::new();
        let processor = ConversationProcessor::new();
        let business_info_repository = restaurant_repository.clone();
        let menu_repository = restaurant_repository.clone();
        let reservation_repository = restaurant_repository.clone();
        let availability_repository = restaurant_repository.clone();
        let restaurant_menu_service = ConversationRestaurantMenuService::new(
            menu_repository,
            Arc::new(business_info_repository.clone()),
        );
        let restaurant_reservation_service = ConversationRestaurantReservationService::new(
            reservation_repository,
            availability_repository,
        );
        let use_case = HandleConversationService::new(
            domain,
            processor,
            analyzer,
            repo.clone(),
            detector.clone(),
            business_id(),
            business_info_repository,
            restaurant_menu_service,
            restaurant_reservation_service,
        );
        UseCaseParts {
            use_case,
            repo,
            detector,
            restaurant_repository,
        }
    }

    fn make_command(message: &str, session_id: Option<&str>) -> HandleConversationCommand {
        HandleConversationCommand {
            message: message.to_string(),
            session_id: session_id.map(str::to_string),
        }
    }

    fn reply_text(reply: &[String]) -> String {
        reply.join("\n")
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

        assert_eq!(
            result.reply,
            vec![
                "Hello. I can help with reservations, menu questions, prices, and restaurant information.".to_string(),
                "Mon-Sun 9am-10pm".to_string()
            ]
        );
        assert_eq!(parts.restaurant_repository.opening_hours_calls(), 1);
    }

    #[test]
    fn active_workflow_uses_derived_task_and_prompts_next_slot() {
        let analyzer = StubNlpAnalyzer::new(vec![
            analysis(
                "reservation_create",
                vec![
                    entity("person", "Jean Martin"),
                    entity("date", "2099-06-12"),
                    entity("time", "7pm"),
                ],
            ),
            analysis("reservation_create", vec![entity("people_count", "4")]),
        ]);
        let parts = make_use_case(analyzer.clone());

        let start = parts.use_case.handle_message(make_command("book", None));
        let next = parts
            .use_case
            .handle_message(make_command("for 4 people", Some(&start.session_id)));

        assert!(reply_text(&start.reply).ends_with("For how many people?"));
        assert!(reply_text(&next.reply).contains("Jean Martin"));
        assert!(reply_text(&next.reply).contains("19:00"));
        assert!(reply_text(&next.reply).contains("4 people"));
        assert!(reply_text(&next.reply).contains("Do you confirm this reservation?"));
        let recorded = analyzer.recorded_tasks();
        assert_eq!(recorded.len(), 2);
        assert!(recorded[0].is_none());
        assert_eq!(recorded[1], Some("WF_RESERVATION_CREATE".to_string()));
        assert_eq!(
            analyzer.recorded_slots(),
            vec![None, Some("people".to_string())]
        );
    }

    #[test]
    fn ready_for_confirmation_uses_choice_task() {
        let analyzer = StubNlpAnalyzer::new(vec![
            analysis(
                "reservation_create",
                vec![
                    entity("person", "Jean Martin"),
                    entity("date", "2099-06-12"),
                    entity("time", "7pm"),
                    entity("people_count", "4"),
                ],
            ),
            analysis("affirmative", vec![]),
        ]);
        let parts = make_use_case(analyzer.clone());

        let start = parts.use_case.handle_message(make_command("book", None));
        let confirm = parts
            .use_case
            .handle_message(make_command("yes", Some(&start.session_id)));

        assert!(reply_text(&start.reply).contains("Jean Martin"));
        assert!(reply_text(&start.reply).contains("19:00"));
        assert!(reply_text(&start.reply).contains("Do you confirm this reservation?"));
        assert!(reply_text(&confirm.reply).contains("Jean Martin"));
        assert!(reply_text(&confirm.reply).contains("19:00"));
        assert!(reply_text(&confirm.reply).contains("REST-000001"));
        let recorded = analyzer.recorded_tasks();
        assert_eq!(recorded.len(), 2);
        assert!(recorded[0].is_none());
        assert_eq!(recorded[1], Some("WF_CHOICE".to_string()));
        assert_eq!(analyzer.recorded_slots(), vec![None, None]);
    }

    #[test]
    fn failed_confirmation_reopens_choice_task_after_slots_are_refilled() {
        let analyzer = StubNlpAnalyzer::new(vec![
            analysis(
                "reservation_create",
                vec![
                    entity("person", "Jean Martin"),
                    entity("date", "2099-06-12"),
                    entity("time", "11pm"),
                    entity("people_count", "4"),
                ],
            ),
            analysis("affirmative", vec![]),
            analysis(
                "reservation_create",
                vec![entity("date", "2099-06-12"), entity("time", "7pm")],
            ),
            analysis("affirmative", vec![]),
        ]);
        let parts = make_use_case(analyzer.clone());

        let start = parts.use_case.handle_message(make_command("book", None));
        let failed_confirm = parts
            .use_case
            .handle_message(make_command("yes", Some(&start.session_id)));
        let refill = parts
            .use_case
            .handle_message(make_command("2099-06-12 7pm", Some(&start.session_id)));
        let success = parts
            .use_case
            .handle_message(make_command("yes", Some(&start.session_id)));

        assert!(reply_text(&start.reply).contains("Do you confirm this reservation?"));
        assert!(reply_text(&failed_confirm.reply).contains("closed"));
        assert!(reply_text(&refill.reply).contains("Do you confirm this reservation?"));
        assert!(reply_text(&success.reply).contains("REST-000001"));

        assert_eq!(
            analyzer.recorded_tasks(),
            vec![
                None,
                Some("WF_CHOICE".to_string()),
                Some("WF_RESERVATION_CREATE".to_string()),
                Some("WF_CHOICE".to_string()),
            ]
        );
    }

    #[test]
    fn cancel_intent_cancels_active_workflow() {
        let analyzer = StubNlpAnalyzer::new(vec![
            analysis("reservation_create", vec![entity("person", "Jean Martin")]),
            analysis("cancel", vec![]),
        ]);
        let parts = make_use_case(analyzer);

        let start = parts.use_case.handle_message(make_command("book", None));
        let cancel = parts
            .use_case
            .handle_message(make_command("cancel", Some(&start.session_id)));

        assert_eq!(reply_text(&cancel.reply), "Okay, I cancelled the current workflow.");
    }

    #[test]
    fn renderer_returns_indonesian_localized_prompt() {
        let analyzer = StubNlpAnalyzer::new(vec![analysis(
            "reservation_create",
            vec![
                entity("person", "Budi Santoso"),
                entity("date", "2099-06-12"),
                entity("time", "jam 7 malam"),
            ],
        )]);
        let parts = make_use_case_for_domain(DomainType::Restaurant, analyzer.clone(), "id");

        let result = parts
            .use_case
            .handle_message(make_command("Halo, saya mau pesan meja", None));

        assert!(reply_text(&result.reply).ends_with("Untuk berapa orang?"));
        assert_eq!(analyzer.recorded_langs(), vec!["id".to_string()]);
    }

    #[test]
    fn injected_domain_is_forwarded_to_nlu() {
        let analyzer = StubNlpAnalyzer::new(vec![analysis("greeting", vec![])]);
        let parts = make_use_case_for_domain(DomainType::Hotel, analyzer.clone(), "en");

        let result = parts.use_case.handle_message(make_command("hello", None));

        assert_eq!(reply_text(&result.reply), "Hello. I can help with reservations, menu questions, prices, and restaurant information.\nDetected intent: greeting");
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
