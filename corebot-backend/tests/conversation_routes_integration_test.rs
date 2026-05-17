use std::sync::Arc;
use std::sync::Mutex;

use axum_test::TestServer;
use serde_json::json;

use corebot_backend::core::conversation::adapter::inbound::web::routes::conversation_routes_with_use_case;
use corebot_backend::core::conversation::adapter::outbound::in_memory_conversation_repository::InMemoryConversationRepository;
use corebot_backend::core::conversation::application::conversation_processor::ConversationProcessor;
use corebot_backend::core::conversation::application::conversation_service::HandleConversationService;
use corebot_backend::core::conversation::application::dto::nlu_analysis_result::{
    NluAnalysisResult, NluEntityResult, NluIntentCandidate,
};
use corebot_backend::core::conversation::application::port::outbound::language_detector_port::LanguageDetectorPort;
use corebot_backend::core::conversation::application::port::outbound::nlp_engine_gateway_port::NlpEngineGatewayPort;
use corebot_backend::core::conversation::application::port::outbound::restaurant::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use corebot_backend::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use corebot_backend::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use corebot_backend::core::conversation::application::port::outbound::restaurant::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
use corebot_backend::core::conversation::application::service::restaurant::{
    ConversationRestaurantMenuService, ConversationRestaurantReservationService,
};
use corebot_backend::core::conversation::domain::restaurant::model::{
    BusinessFact, BusinessLocation, ContactChannel, EventSpace, Facility, MenuItem, MenuPriceFilter,
    OpeningHours, PaymentMethod, Reservation, ReservationDraft, ReservationSettings,
    RestaurantRepositoryError, TableType,
};
use corebot_backend::core::conversation::domain::domain_type::DomainType;
use corebot_backend::core::conversation::domain::model::intent::NluTask;
use corebot_backend::core::conversation::domain::model::slot::SlotName;
use chrono::{NaiveDate, NaiveTime, Weekday};
use uuid::Uuid;

struct StubNlpAnalyzer {
    intent_name: &'static str,
}

impl NlpEngineGatewayPort for StubNlpAnalyzer {
    fn analyze(
        &self,
        text: &str,
        lang: &str,
        domain: DomainType,
        task: Option<NluTask>,
        slot_hint: Option<SlotName>,
    ) -> NluAnalysisResult {
        let _ = (lang, domain, task, slot_hint, text);
        NluAnalysisResult {
            intent_name: self.intent_name.to_string(),
            intent_confidence: 1.0,
            intent_candidates: vec![],
            entities: vec![],
        }
    }
}

#[derive(Clone)]
struct ScriptedNlpAnalyzer {
    responses: Arc<Mutex<Vec<NluAnalysisResult>>>,
}

impl ScriptedNlpAnalyzer {
    fn new(responses: Vec<NluAnalysisResult>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses.into_iter().rev().collect())),
        }
    }
}

impl NlpEngineGatewayPort for ScriptedNlpAnalyzer {
    fn analyze(
        &self,
        _text: &str,
        _lang: &str,
        _domain: DomainType,
        _task: Option<NluTask>,
        _slot_hint: Option<SlotName>,
    ) -> NluAnalysisResult {
        self.responses
            .lock()
            .unwrap()
            .pop()
            .expect("missing scripted NLU response")
    }
}

struct StubLanguageDetector;

impl LanguageDetectorPort for StubLanguageDetector {
    fn detect(&self, _text: &str) -> String {
        "en".to_string()
    }
}

#[derive(Clone)]
struct FakeRepository {
    reservations: Arc<Mutex<Vec<Reservation>>>,
}

impl FakeRepository {
    fn new() -> Self {
        Self {
            reservations: Arc::new(Mutex::new(vec![])),
        }
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

fn restaurant_parts(
    repository: FakeRepository,
) -> (
    Uuid,
    FakeRepository,
    ConversationRestaurantMenuService<FakeRepository>,
    ConversationRestaurantReservationService<FakeRepository, FakeRepository>,
) {
    let business_info_repository = repository.clone();
    let menu_repository = repository.clone();
    let reservation_repository = repository.clone();
    let availability_repository = repository;
    let menu_business_info_repository = business_info_repository.clone();
    (
        business_id(),
        business_info_repository,
        ConversationRestaurantMenuService::new(
            menu_repository,
            Arc::new(menu_business_info_repository),
        ),
        ConversationRestaurantReservationService::new(
            reservation_repository,
            availability_repository,
        ),
    )
}

#[async_trait::async_trait]
impl RestaurantBusinessInfoRepositoryPort for FakeRepository {
    async fn opening_hours(&self, _: Uuid) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
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
        Ok(vec![])
    }

    async fn payment_methods(
        &self,
        _: Uuid,
    ) -> Result<Vec<PaymentMethod>, RestaurantRepositoryError> {
        Ok(vec![])
    }

    async fn facilities(&self, _: Uuid) -> Result<Vec<Facility>, RestaurantRepositoryError> {
        Ok(vec![])
    }

    async fn facts(
        &self,
        _: Uuid,
        _: &str,
    ) -> Result<Vec<BusinessFact>, RestaurantRepositoryError> {
        Ok(vec![])
    }

    async fn event_spaces(&self, _: Uuid) -> Result<Vec<EventSpace>, RestaurantRepositoryError> {
        Ok(vec![])
    }
}

#[async_trait::async_trait]
impl RestaurantMenuRepositoryPort for FakeRepository {
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
        _: &MenuPriceFilter,
    ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
        Ok(vec![])
    }
}

#[async_trait::async_trait]
impl RestaurantReservationRepositoryPort for FakeRepository {
    async fn next_reference_index(&self, _: Uuid) -> Result<i64, RestaurantRepositoryError> {
        Ok(self.reservations.lock().unwrap().len() as i64 + 11)
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
impl RestaurantAvailabilityRepositoryPort for FakeRepository {
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

    async fn opening_hours(&self, _: Uuid) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
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

fn make_server(intent_name: &'static str) -> TestServer {
    let processor = ConversationProcessor::new();
    let analyzer = StubNlpAnalyzer { intent_name };
    let repository = InMemoryConversationRepository::new();
    let language_detector = StubLanguageDetector;
    let (
        restaurant_context,
        business_info_repository,
        restaurant_menu_service,
        restaurant_reservation_service,
    ) = restaurant_parts(FakeRepository::new());
    let use_case = Arc::new(HandleConversationService::new(
        DomainType::Restaurant,
        processor,
        analyzer,
        repository,
        language_detector,
        restaurant_context,
        business_info_repository,
        restaurant_menu_service,
        restaurant_reservation_service,
    ));
    TestServer::new(conversation_routes_with_use_case(use_case))
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

fn make_scripted_server(responses: Vec<NluAnalysisResult>) -> TestServer {
    let (
        restaurant_context,
        business_info_repository,
        restaurant_menu_service,
        restaurant_reservation_service,
    ) = restaurant_parts(FakeRepository::new());
    let processor = ConversationProcessor::new();
    let analyzer = ScriptedNlpAnalyzer::new(responses);
    let repository = InMemoryConversationRepository::new();
    let language_detector = StubLanguageDetector;
    let use_case = Arc::new(HandleConversationService::new(
        DomainType::Restaurant,
        processor,
        analyzer,
        repository,
        language_detector,
        restaurant_context,
        business_info_repository,
        restaurant_menu_service,
        restaurant_reservation_service,
    ));
    TestServer::new(conversation_routes_with_use_case(use_case))
}

#[tokio::test]
async fn post_send_message_returns_200_with_session_id() {
    let server = make_server("ask_opening_hours");

    let response = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "hello" }))
        .await;

    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert!(
        body["session_id"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert_eq!(body["reply"], "Mon-Sun 9am-10pm");
}

#[tokio::test]
async fn post_send_message_reuses_provided_session_id() {
    let server = make_server("greeting");
    let session_id = uuid::Uuid::new_v4().to_string();

    let response = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "hello", "session_id": session_id }))
        .await;

    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["session_id"], session_id);
}

#[tokio::test]
async fn post_send_message_returns_415_when_content_type_missing() {
    let server = make_server("unknown");

    let response = server.post("/api/v1/conversation/send_message").await;

    response.assert_status(axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn post_send_message_returns_422_when_message_field_missing() {
    let server = make_server("unknown");

    let response = server
        .post("/api/v1/conversation/send_message")
        .json(&serde_json::json!({ "session_id": "abc" }))
        .await;

    response.assert_status_unprocessable_entity();
}

#[tokio::test]
async fn multi_turn_reservation_flow_returns_summary_reference_and_supports_check_without_reference()
 {
    let server = make_scripted_server(vec![
        analysis("greeting", vec![]),
        analysis("reservation_create", vec![]),
        analysis("reservation_create", vec![entity("person", "Stephane")]),
        analysis(
            "reservation_create",
            vec![entity("date", "tomorrow"), entity("time", "7pm")],
        ),
        analysis("reservation_create", vec![entity("people_count", "4")]),
        analysis("affirmative", vec![]),
        analysis("check_reservation", vec![]),
        analysis("ask_location", vec![]),
    ]);

    let hello = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "hello" }))
        .await
        .json::<serde_json::Value>();
    let session_id = hello["session_id"].as_str().unwrap().to_string();
    assert_eq!(
        hello["reply"],
        "Hello! How can I help with the restaurant today?"
    );

    let start = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "i want to book", "session_id": session_id.clone() }))
        .await
        .json::<serde_json::Value>();
    // starting_message is prepended — just verify the slot prompt is included
    assert!(
        start["reply"]
            .as_str()
            .unwrap()
            .contains("What name should I use for the reservation?")
    );

    let name = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "Stephane", "session_id": session_id.clone() }))
        .await
        .json::<serde_json::Value>();
    assert_eq!(name["reply"], "What date would you like?");

    let date_time = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "tomorrow at 7pm", "session_id": session_id.clone() }))
        .await
        .json::<serde_json::Value>();
    assert_eq!(date_time["reply"], "For how many people?");

    let people = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "4", "session_id": session_id.clone() }))
        .await
        .json::<serde_json::Value>();
    let people_reply = people["reply"].as_str().unwrap();
    assert!(people_reply.contains("Stephane"));
    assert!(people_reply.contains("19:00"));
    assert!(people_reply.contains("4 people"));
    assert!(people_reply.contains("Do you confirm this reservation?"));

    let confirm = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "yes", "session_id": session_id.clone() }))
        .await
        .json::<serde_json::Value>();
    let confirmation_reply = confirm["reply"].as_str().unwrap();
    assert!(confirmation_reply.contains("Your reservation is confirmed for Stephane"));
    assert!(confirmation_reply.contains("19:00"));
    assert!(confirmation_reply.contains("4 people"));
    assert!(confirmation_reply.contains("Your reference is REST-00000B."));

    let check = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "look up my booking", "session_id": session_id.clone() }))
        .await
        .json::<serde_json::Value>();
    let check_reply = check["reply"].as_str().unwrap();
    assert!(check_reply.contains("I found these reservations under Stephane:"));
    assert!(check_reply.contains("REST-00000B"));

    let location = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "where are you localized", "session_id": session_id }))
        .await
        .json::<serde_json::Value>();
    assert!(
        location["reply"]
            .as_str()
            .unwrap()
            .contains("12 Rue de la Paix")
    );
}
