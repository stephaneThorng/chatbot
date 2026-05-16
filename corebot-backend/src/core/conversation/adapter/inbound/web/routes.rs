use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};

use super::send_message_dto::{SendMessageRequest, SendMessageResponse};
use crate::core::conversation::application::port::inbound::conversation_usecase::HandleConversationUseCase;

async fn send_message<U>(
    State(usecase): State<Arc<U>>,
    Json(request): Json<SendMessageRequest>,
) -> Json<SendMessageResponse>
where
    U: HandleConversationUseCase + Send + Sync + 'static,
{
    let result = usecase.handle_message(request.into()).await;
    Json(result.into())
}

pub fn conversation_routes_with_use_case<U>(use_case: Arc<U>) -> Router
where
    U: HandleConversationUseCase + Send + Sync + 'static,
{
    Router::new()
        .route("/api/v1/conversation/send_message", post(send_message::<U>))
        .with_state(use_case)
}
