use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};

use super::send_message_dto::{SendMessageRequest, SendMessageResponse};
use crate::core::conversation::application::conversation_usecase::HandleConversationUseCase;
use crate::core::conversation::application::port::input::conversation_trait::HandleConversation;

async fn send_message(
    State(use_case): State<Arc<dyn HandleConversation + Send + Sync>>,
    Json(request): Json<SendMessageRequest>,
) -> Json<SendMessageResponse> {
    let result = use_case.handle_message(request.into());
    Json(result.into())
}

pub fn conversation_routes() -> Router {
    let use_case: Arc<dyn HandleConversation + Send + Sync> = Arc::new(HandleConversationUseCase);

    Router::new()
        .route("/api/v1/conversation/send_message", post(send_message))
        .with_state(use_case)
}
