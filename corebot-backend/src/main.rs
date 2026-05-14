use std::sync::Arc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use corebot_backend::core::conversation::adapter::inbound::web::routes::conversation_routes_with_use_case;
use corebot_backend::core::conversation::adapter::outbound::english_date_resolver::EnglishDateResolver;
use corebot_backend::core::conversation::adapter::outbound::in_memory_conversation_repository::InMemoryConversationRepository;
use corebot_backend::core::conversation::adapter::outbound::langdetect_language_detector::LangdetectLanguageDetector;
use corebot_backend::core::conversation::adapter::outbound::nlu_engine_gateway::NluEngineGateway;
use corebot_backend::core::conversation::adapter::outbound::restaurant_domain_gateway::{
    RestaurantInformationGateway, RestaurantReservationGateway,
};
use corebot_backend::core::conversation::application::conversation_processor::ConversationProcessor;
use corebot_backend::core::conversation::application::conversation_service::HandleConversationService;
use corebot_backend::core::conversation::application::intent_handler::IntentHandlerRegistry;
use corebot_backend::core::conversation::application::restaurant_handler_registry_factory::{
    RestaurantConversationDependencies, RestaurantHandlerRegistryFactory,
};
use corebot_backend::core::conversation::domain::domain_type::DomainType;
use corebot_backend::core::nlu_engine::adapter::outbound::onnx_nlu_runtime::OnnxNluRuntime;
use corebot_backend::core::nlu_engine::application::AnalyzeTextService;
use corebot_backend::core::restaurant::adapter::inbound::restaurant_adapter::RestaurantAdapter;

const BIND_ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let restaurant = Arc::new(RestaurantAdapter::new());
    let date_resolver = Arc::new(EnglishDateResolver);
    let restaurant_registry =
        RestaurantHandlerRegistryFactory::build(RestaurantConversationDependencies {
            information_port: Arc::new(RestaurantInformationGateway::new(Arc::clone(&restaurant))),
            reservation_port: Arc::new(RestaurantReservationGateway::new(Arc::clone(&restaurant))),
            date_resolver,
        });
    let processor =
        ConversationProcessor::new(restaurant_registry, IntentHandlerRegistry::new(vec![]));

    let runtime = OnnxNluRuntime::from_env()
        .unwrap_or_else(|error| panic!("Failed to initialize ONNX NLU runtime: {error}"));
    let nlu_use_case = AnalyzeTextService::new(runtime);
    let analyzer = NluEngineGateway::new(nlu_use_case);
    let conversation_repository = InMemoryConversationRepository::new();
    let language_detector = LangdetectLanguageDetector::new();
    let use_case = Arc::new(HandleConversationService::new(
        DomainType::Restaurant,
        processor,
        analyzer,
        conversation_repository,
        language_detector,
    ));
    let app = Router::new()
        .merge(conversation_routes_with_use_case(use_case))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(BIND_ADDRESS)
        .await
        .unwrap_or_else(|error| panic!("Failed to bind to {BIND_ADDRESS}: {error}"));

    println!("Server listening on {BIND_ADDRESS}");

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|error| panic!("Server error: {error}"));
}
