use std::sync::Arc;

use axum_test::TestServer;
use serde_json::json;

use corebot_backend::core::conversation::adapter::inbound::web::routes::conversation_routes_with_use_case;
use corebot_backend::core::conversation::adapter::outbound::in_memory_conversation_repository::InMemoryConversationRepository;
use corebot_backend::core::conversation::adapter::outbound::restaurant_domain_gateway::RestaurantDomainGateway;
use corebot_backend::core::conversation::application::conversation_service::HandleConversationService;
use corebot_backend::core::conversation::application::port::outbound::language_detector_port::LanguageDetectorPort;
use corebot_backend::core::conversation::application::port::outbound::nlp_engine_gateway_port::NlpEngineGatewayPort;
use corebot_backend::core::conversation::domain::domain_type::DomainType;
use corebot_backend::core::conversation::domain::model::intent::NluTask;
use corebot_backend::core::nlu_engine::domain::analysis::{NluAnalysis, NluIntent};
use corebot_backend::core::restaurant::application::port::inbound::restaurant_trait::RestaurantPort;

struct StubRestaurantPort;

impl RestaurantPort for StubRestaurantPort {
    fn get_opening_hours(&self) -> String {
        "Not implemented yet".to_string()
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

struct StubLanguageDetector;

impl LanguageDetectorPort for StubLanguageDetector {
    fn detect(&self, _text: &str) -> String {
        "en".to_string()
    }
}

fn make_server(intent_name: &'static str) -> TestServer {
    let gateway = RestaurantDomainGateway::new(StubRestaurantPort);
    let analyzer = StubNlpAnalyzer { intent_name };
    let repository = InMemoryConversationRepository::new();
    let language_detector = StubLanguageDetector;
    let use_case = Arc::new(HandleConversationService::new(
        DomainType::Restaurant,
        gateway,
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
    assert!(body["session_id"].as_str().is_some_and(|s| !s.is_empty()));
    assert_eq!(body["reply"], "Not implemented yet");
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
