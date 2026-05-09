use axum::Router;
use std::sync::Arc;

use corebot_backend::core::conversation::adapter::input::web::routes::conversation_routes_with_use_case;
use corebot_backend::core::conversation::adapter::output::nlu_engine_gateway::NluEngineGateway;
use corebot_backend::core::conversation::adapter::output::restaurant_domain_gateway::RestaurantDomainGateway;
use corebot_backend::core::conversation::application::conversation_usecase::HandleConversationUseCase;
use corebot_backend::core::conversation::application::port::input::conversation_trait::HandleConversation;
use corebot_backend::core::conversation::application::port::output::nlp_analyzer_trait::NlpEngineGatewayPort;
use corebot_backend::core::nlu_engine::adapter::output::onnx_nlu_runtime::OnnxNluRuntime;
use corebot_backend::core::nlu_engine::application::AnalyzeTextUseCase;
use corebot_backend::core::nlu_engine::application::port::input::analyze_text_trait::AnalyzeText;
use corebot_backend::core::nlu_engine::application::port::output::nlu_model_runtime_trait::NluModelRuntime;
use corebot_backend::core::restaurant::adapter::input::restaurant_adapter::RestaurantAdapter;

const BIND_ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    let gateway = Arc::new(RestaurantDomainGateway::new(Arc::new(RestaurantAdapter)));
    let runtime: Arc<dyn NluModelRuntime> = Arc::new(
        OnnxNluRuntime::from_env()
            .unwrap_or_else(|error| panic!("Failed to initialize ONNX NLU runtime: {error}")),
    );
    let nlu_use_case: Arc<dyn AnalyzeText> = Arc::new(AnalyzeTextUseCase::new(runtime));
    let analyzer: Arc<dyn NlpEngineGatewayPort> = Arc::new(NluEngineGateway::new(nlu_use_case));
    let use_case: Arc<dyn HandleConversation + Send + Sync> =
        Arc::new(HandleConversationUseCase::new(gateway, analyzer));
    let app = Router::new().merge(conversation_routes_with_use_case(use_case));

    let listener = tokio::net::TcpListener::bind(BIND_ADDRESS)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {BIND_ADDRESS}: {e}"));

    println!("Server listening on {BIND_ADDRESS}");

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("Server error: {e}"));
}
