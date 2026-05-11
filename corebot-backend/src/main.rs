use axum::Router;
use std::sync::Arc;

use corebot_backend::core::conversation::adapter::inbound::web::routes::conversation_routes_with_use_case;
use corebot_backend::core::conversation::adapter::outbound::in_memory_conversation_repository::InMemoryConversationRepository;
use corebot_backend::core::conversation::adapter::outbound::langdetect_language_detector::LangdetectLanguageDetector;
use corebot_backend::core::conversation::adapter::outbound::nlu_engine_gateway::NluEngineGateway;
use corebot_backend::core::conversation::adapter::outbound::restaurant_domain_gateway::RestaurantDomainGateway;
use corebot_backend::core::conversation::application::conversation_service::HandleConversationService;
use corebot_backend::core::conversation::domain::domain_type::DomainType;
use corebot_backend::core::nlu_engine::adapter::outbound::onnx_nlu_runtime::OnnxNluRuntime;
use corebot_backend::core::nlu_engine::application::AnalyzeTextService;
use corebot_backend::core::restaurant::adapter::inbound::restaurant_adapter::RestaurantAdapter;

const BIND_ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    let gateway = RestaurantDomainGateway::new(RestaurantAdapter);
    let runtime = OnnxNluRuntime::from_env()
        .unwrap_or_else(|error| panic!("Failed to initialize ONNX NLU runtime: {error}"));
    let nlu_use_case = AnalyzeTextService::new(runtime);
    let analyzer = NluEngineGateway::new(nlu_use_case);
    let conversation_repository = InMemoryConversationRepository::new();
    let language_detector = LangdetectLanguageDetector::new();
    let use_case = Arc::new(HandleConversationService::new(
        DomainType::Restaurant,
        gateway,
        analyzer,
        conversation_repository,
        language_detector,
    ));
    let app = Router::new().merge(conversation_routes_with_use_case(use_case));

    let listener = tokio::net::TcpListener::bind(BIND_ADDRESS)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {BIND_ADDRESS}: {e}"));

    println!("Server listening on {BIND_ADDRESS}");

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("Server error: {e}"));
}
