use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};

use super::send_message_dto::{SendMessageRequest, SendMessageResponse};
use crate::core::conversation::adapter::output::restaurant_domain_gateway::RestaurantDomainGateway;
use crate::core::conversation::application::conversation_usecase::HandleConversationUseCase;
use crate::core::conversation::application::port::input::conversation_trait::HandleConversation;
use crate::core::restaurant::adapter::input::restaurant_adapter::RestaurantAdapter;

async fn send_message(
    State(usecase): State<Arc<dyn HandleConversation + Send + Sync>>,
    Json(request): Json<SendMessageRequest>,
) -> Json<SendMessageResponse> {
    let result = usecase.handle_message(request.into());
    Json(result.into())
}

pub fn conversation_routes() -> Router {
    // The domain is known because he is related to the called endpoint
    let gateway = Arc::new(RestaurantDomainGateway::new(Arc::new(RestaurantAdapter)));
    let use_case: Arc<dyn HandleConversation + Send + Sync> =
        Arc::new(HandleConversationUseCase::new(gateway));

    Router::new()
        .route("/api/v1/conversation/send_message", post(send_message))
        .with_state(use_case)
}
