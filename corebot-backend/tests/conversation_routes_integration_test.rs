use std::sync::Arc;

use axum_test::TestServer;
use serde_json::json;

use corebot_backend::core::conversation::adapter::input::web::routes::conversation_routes_with_use_case;
use corebot_backend::core::conversation::adapter::output::restaurant_domain_gateway::RestaurantDomainGateway;
use corebot_backend::core::conversation::application::conversation_usecase::HandleConversationUseCase;
use corebot_backend::core::conversation::application::port::input::conversation_trait::HandleConversation;
use corebot_backend::core::conversation::application::port::output::nlp_analyzer_trait::NlpEngineGatewayPort;
use corebot_backend::core::nlu_engine::domain::analysis::{NluAnalysis, NluIntent};
use corebot_backend::core::restaurant::application::port::input::restaurant_trait::RestaurantPort;

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
    fn analyze(&self, text: &str, lang: &str, domain: &str, task: Option<String>) -> NluAnalysis {
        let _ = (lang, domain, task);
        NluAnalysis {
            tagged_text: text.to_string(),
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

fn make_server(intent_name: &'static str) -> TestServer {
    let gateway = Arc::new(RestaurantDomainGateway::new(Arc::new(StubRestaurantPort)));
    let analyzer = Arc::new(StubNlpAnalyzer { intent_name });
    let use_case: Arc<dyn HandleConversation + Send + Sync> =
        Arc::new(HandleConversationUseCase::new(gateway, analyzer));
    TestServer::new(conversation_routes_with_use_case(use_case))
}

#[tokio::test]
async fn post_send_message_returns_200_with_session_id() {
    let server = make_server("opening_hours");

    let response = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "hello" }))
        .await;

    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert!(body["session_id"].as_str().is_some_and(|s| !s.is_empty()));
    assert_eq!(body["reply"], "Not implemented yet");
    assert_eq!(body["detected_intent"], "opening_hours");
}

#[tokio::test]
async fn post_send_message_reuses_provided_session_id() {
    let server = make_server("greeting");

    let response = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "hello", "session_id": "my-session-42" }))
        .await;

    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["session_id"], "my-session-42");
    assert_eq!(body["detected_intent"], "greeting");
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
