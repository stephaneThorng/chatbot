use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use corebot_backend::core::conversation::adapter::inbound::web::routes::conversation_routes_with_use_case;
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
use corebot_backend::core::restaurant::application::restaurant_service::RestaurantService;

const BIND_ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    log_environment(load_environment());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let restaurant = Arc::new(RestaurantService::new());
    let information_gateway = Box::leak(Box::new(RestaurantInformationGateway::new(
        Arc::clone(&restaurant),
    )));
    let reservation_gateway = Box::leak(Box::new(RestaurantReservationGateway::new(
        Arc::clone(&restaurant),
    )));
    let restaurant_registry =
        RestaurantHandlerRegistryFactory::build(RestaurantConversationDependencies {
            information_port: information_gateway,
            reservation_port: reservation_gateway,
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

fn load_environment() -> Result<Option<PathBuf>, String> {
    if let Ok(path) = dotenvy::dotenv() {
        return Ok(Some(path));
    }

    let explicit_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    match dotenvy::from_path(&explicit_path) {
        Ok(()) => Ok(Some(explicit_path)),
        Err(dotenv_error) if explicit_path.exists() => Err(format!(
            "failed to parse {}: {}",
            explicit_path.display(),
            dotenv_error
        )),
        Err(_) => Ok(None),
    }
}

fn log_environment(env_result: Result<Option<PathBuf>, String>) {
    match env_result {
        Ok(Some(path)) => println!("[startup] loaded env from {}", path.display()),
        Ok(None) => println!("[startup] no .env file loaded"),
        Err(error) => println!("[startup] {}", error),
    }

    println!(
        "[startup] COREBOT_DEBUG_NLU={}",
        if debug_nlu_logging_enabled() {
            "on"
        } else {
            "off"
        }
    );
    println!(
        "[startup] COREBOT_NLU_ONNX_DIR={}",
        std::env::var("COREBOT_NLU_ONNX_DIR").unwrap_or_else(|_| "<unset>".to_string())
    );
}

fn debug_nlu_logging_enabled() -> bool {
    std::env::var("COREBOT_DEBUG_NLU")
        .ok()
        .as_deref()
        .map(is_truthy_env_value)
        .unwrap_or(false)
}

fn is_truthy_env_value(value: &str) -> bool {
    let normalized = value.trim().trim_matches('\'').trim_matches('"');
    matches!(
        normalized.to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}
