use axum_test::TestServer;
use serde_json::json;

use corebot_backend::core::conversation::adapter::input::web::routes::conversation_routes;

fn make_server() -> TestServer {
    TestServer::new(conversation_routes())
}

#[tokio::test]
async fn post_send_message_returns_200_with_session_id() {
    let server = make_server();

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
    let server = make_server();

    let response = server
        .post("/api/v1/conversation/send_message")
        .json(&json!({ "message": "hello", "session_id": "my-session-42" }))
        .await;

    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["session_id"], "my-session-42");
}

#[tokio::test]
async fn post_send_message_returns_415_when_content_type_missing() {
    let server = make_server();

    let response = server
        .post("/api/v1/conversation/send_message")
        .await;

    response.assert_status(axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn post_send_message_returns_422_when_message_field_missing() {
    let server = make_server();

    let response = server
        .post("/api/v1/conversation/send_message")
        .json(&serde_json::json!({ "session_id": "abc" }))
        .await;

    response.assert_status_unprocessable_entity();
}



