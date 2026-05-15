use std::sync::Arc;
use std::sync::Mutex;

use axum_test::TestServer;
use serde_json::json;

use corebot_backend::core::conversation::adapter::inbound::web::routes::conversation_routes_with_use_case;
use corebot_backend::core::conversation::adapter::outbound::in_memory_conversation_repository::InMemoryConversationRepository;
use corebot_backend::core::conversation::adapter::outbound::restaurant_domain_gateway::{
    RestaurantInformationGateway, RestaurantReservationGateway,
};
use corebot_backend::core::conversation::application::conversation_processor::ConversationProcessor;
use corebot_backend::core::conversation::application::conversation_service::HandleConversationService;
use corebot_backend::core::conversation::application::intent_handler::IntentHandlerRegistry;
use corebot_backend::core::conversation::application::port::outbound::language_detector_port::LanguageDetectorPort;
use corebot_backend::core::conversation::application::port::outbound::nlp_engine_gateway_port::NlpEngineGatewayPort;
use corebot_backend::core::conversation::application::restaurant_handler_registry_factory::{
    RestaurantConversationDependencies, RestaurantHandlerRegistryFactory,
};
use corebot_backend::core::conversation::domain::domain_type::DomainType;
use corebot_backend::core::conversation::domain::model::intent::NluTask;
use corebot_backend::core::conversation::domain::slot::EntityType;
use corebot_backend::core::nlu_engine::domain::analysis::{
    NerTokenLabel, NluAnalysis, NluEntity, NluIntent, NluIntentCandidate,
};
use corebot_backend::core::restaurant::application::restaurant_service::RestaurantService;
use corebot_backend::core::restaurant::application::port::inbound::restaurant_information_port::RestaurantInformationUseCase;
use corebot_backend::core::restaurant::application::port::inbound::restaurant_queries::{
    EventQuery, FacilityQuery, LocationQuery, MenuDietaryQuery, MenuItemDetailsQuery, MenuQuery,
    PaymentMethodQuery, PriceQuery, ReservationCreateQuery, ReservationLookupQuery,
};
use corebot_backend::core::restaurant::application::port::inbound::restaurant_reservation_port::RestaurantReservationUseCase;

struct StubRestaurant;

impl RestaurantInformationUseCase for StubRestaurant {
    fn get_opening_hours(&self) -> String {
        "Mon-Sun 9am-10pm".to_string()
    }

    fn find_menu(&self, _: MenuQuery) -> String {
        "full_menu:stub".to_string()
    }

    fn find_menu_dietary(&self, _: MenuDietaryQuery) -> String {
        "dietary_no_filter:".to_string()
    }

    fn find_menu_item_details(&self, _: MenuItemDetailsQuery) -> String {
        "details_no_filter:".to_string()
    }

    fn find_location(&self, _: LocationQuery) -> String {
        "address:stub".to_string()
    }

    fn get_contact(&self) -> String {
        "contact:+33123456789|test@example.com".to_string()
    }

    fn find_payment_methods(&self, _: PaymentMethodQuery) -> String {
        "all_methods:cash".to_string()
    }

    fn find_price(&self, _: PriceQuery) -> String {
        "price_general:stub".to_string()
    }

    fn get_takeaway_info(&self) -> String {
        "takeaway:yes|stub".to_string()
    }

    fn find_event_info(&self, _: EventQuery) -> String {
        "event_info:stub".to_string()
    }

    fn find_facility_info(&self, _: FacilityQuery) -> String {
        "all_facilities:wifi".to_string()
    }

    fn get_accessibility_info(&self) -> String {
        "accessibility:yes|stub".to_string()
    }

    fn get_entertainment_info(&self) -> String {
        "entertainment:yes|stub".to_string()
    }
}

impl RestaurantReservationUseCase for StubRestaurant {
    fn create_reservation(&self, _: ReservationCreateQuery) -> Result<String, corebot_backend::core::restaurant::domain::model::ReservationError> {
        Ok("created:REST-NEW123".to_string())
    }

    fn check_reservation(&self, _: ReservationLookupQuery) -> String {
        "no_reference_or_name:".to_string()
    }
}

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
    ) -> NluAnalysis {
        let _ = (lang, domain, task);
        NluAnalysis {
            processed_text: text.to_string(),
            intent: NluIntent {
                name: self.intent_name.to_string(),
                confidence: 1.0,
            },
            intents: vec![],
            entities: vec![],
            ner_labels: vec![],
        }
    }
}

#[derive(Clone)]
struct ScriptedNlpAnalyzer {
    responses: Arc<Mutex<Vec<NluAnalysis>>>,
}

impl ScriptedNlpAnalyzer {
    fn new(responses: Vec<NluAnalysis>) -> Self {
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
    ) -> NluAnalysis {
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

fn make_server(intent_name: &'static str) -> TestServer {
    let restaurant = Arc::new(StubRestaurant);
    let information_gateway = Box::leak(Box::new(RestaurantInformationGateway::new(
        Arc::clone(&restaurant),
    )));
    let reservation_gateway = Box::leak(Box::new(RestaurantReservationGateway::new(
        Arc::clone(&restaurant),
    )));
    let restaurant_registry =
        RestaurantHandlerRegistryFactory::build(RestaurantConversationDependencies {
            information_port: information_gateway,
            reservation_port: reservation_gateway,
        });
    let processor =
        ConversationProcessor::new(restaurant_registry, IntentHandlerRegistry::new(vec![]));
    let analyzer = StubNlpAnalyzer { intent_name };
    let repository = InMemoryConversationRepository::new();
    let language_detector = StubLanguageDetector;
    let use_case = Arc::new(HandleConversationService::new(
        DomainType::Restaurant,
        processor,
        analyzer,
        repository,
        language_detector,
    ));
    TestServer::new(conversation_routes_with_use_case(use_case))
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

fn make_scripted_server(responses: Vec<NluAnalysis>) -> TestServer {
    let restaurant = Arc::new(RestaurantService::new());
    let information_gateway = Box::leak(Box::new(RestaurantInformationGateway::new(
        Arc::clone(&restaurant),
    )));
    let reservation_gateway = Box::leak(Box::new(RestaurantReservationGateway::new(
        Arc::clone(&restaurant),
    )));
    let restaurant_registry =
        RestaurantHandlerRegistryFactory::build(RestaurantConversationDependencies {
            information_port: information_gateway,
            reservation_port: reservation_gateway,
        });
    let processor =
        ConversationProcessor::new(restaurant_registry, IntentHandlerRegistry::new(vec![]));
    let analyzer = ScriptedNlpAnalyzer::new(responses);
    let repository = InMemoryConversationRepository::new();
    let language_detector = StubLanguageDetector;
    let use_case = Arc::new(HandleConversationService::new(
        DomainType::Restaurant,
        processor,
        analyzer,
        repository,
        language_detector,
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
        analysis(
            "reservation_create",
            vec![entity(EntityType::Person, "Stephane")],
        ),
        analysis(
            "reservation_create",
            vec![
                entity(EntityType::Date, "tomorrow"),
                entity(EntityType::Time, "7pm"),
            ],
        ),
        analysis(
            "reservation_create",
            vec![entity(EntityType::PeopleCount, "4")],
        ),
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
    assert!(start["reply"]
        .as_str()
        .unwrap()
        .contains("What name should I use for the reservation?"));

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
    assert!(people_reply.contains("7pm"));
    assert!(people_reply.contains("4 people"));
    assert!(people_reply.contains("Do you confirm this reservation?"));

    let confirm = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "yes", "session_id": session_id.clone() }))
        .await
        .json::<serde_json::Value>();
    let confirmation_reply = confirm["reply"].as_str().unwrap();
    assert!(confirmation_reply.contains("Your reservation is confirmed for Stephane"));
    assert!(confirmation_reply.contains("7pm"));
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
