use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};

use super::send_message_dto::{SendMessageRequest, SendMessageResponse};
use crate::core::conversation::adapter::output::nlu_engine_analyzer::NluEngineAnalyzer;
use crate::core::conversation::adapter::output::restaurant_domain_gateway::RestaurantDomainGateway;
use crate::core::conversation::application::conversation_usecase::HandleConversationUseCase;
use crate::core::conversation::application::port::input::conversation_trait::HandleConversation;
use crate::core::conversation::application::port::output::nlp_analyzer_trait::NlpAnalyzer;
use crate::core::nlu_engine::adapter::output::onnx_nlu_runtime::OnnxNluRuntime;
use crate::core::nlu_engine::application::AnalyzeTextUseCase;
use crate::core::nlu_engine::application::port::input::analyze_text_trait::AnalyzeTextNlu;
use crate::core::nlu_engine::application::port::output::nlu_model_runtime_trait::NluModelRuntime;
use crate::core::restaurant::adapter::input::restaurant_adapter::RestaurantAdapter;

async fn send_message(
    State(usecase): State<Arc<dyn HandleConversation + Send + Sync>>,
    Json(request): Json<SendMessageRequest>,
) -> Json<SendMessageResponse> {
    let result = usecase.handle_message(request.into());
    Json(result.into())
}

pub fn conversation_routes_with_use_case(
    use_case: Arc<dyn HandleConversation + Send + Sync>,
) -> Router {
    Router::new()
        .route("/api/v1/conversation/send_message", post(send_message))
        .with_state(use_case)
}

pub fn conversation_routes() -> Router {
    let gateway = Arc::new(RestaurantDomainGateway::new(Arc::new(RestaurantAdapter)));
    let runtime: Arc<dyn NluModelRuntime> = Arc::new(
        OnnxNluRuntime::from_env()
            .unwrap_or_else(|error| panic!("Failed to initialize ONNX NLU runtime: {error}")),
    );
    let nlu_use_case: Arc<dyn AnalyzeTextNlu> = Arc::new(AnalyzeTextUseCase::new(runtime));
    let analyzer: Arc<dyn NlpAnalyzer> = Arc::new(NluEngineAnalyzer::new(nlu_use_case));
    let use_case: Arc<dyn HandleConversation + Send + Sync> =
        Arc::new(HandleConversationUseCase::new(gateway, analyzer));
    conversation_routes_with_use_case(use_case)
}
