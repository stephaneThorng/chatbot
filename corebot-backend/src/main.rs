use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use corebot_backend::core::conversation::adapter::inbound::web::routes::conversation_routes_with_use_case;
use corebot_backend::core::conversation::adapter::outbound::in_memory_conversation_repository::InMemoryConversationRepository;
use corebot_backend::core::conversation::adapter::outbound::langdetect_language_detector::LangdetectLanguageDetector;
use corebot_backend::core::conversation::adapter::outbound::nlu_engine_gateway::NluEngineGateway;
use corebot_backend::core::conversation::adapter::outbound::restaurant_business_info_gateway::RestaurantBusinessInfoGateway;
use corebot_backend::core::conversation::adapter::outbound::restaurant_menu_gateway::RestaurantMenuGateway;
use corebot_backend::core::conversation::adapter::outbound::restaurant_reservation_gateway::RestaurantReservationGateway;
use corebot_backend::core::conversation::application::conversation_processor::ConversationProcessor;
use corebot_backend::core::conversation::application::conversation_service::HandleConversationService;
use corebot_backend::core::conversation::domain::domain_type::DomainType;
use corebot_backend::core::nlu_engine::adapter::outbound::onnx_nlu_runtime::OnnxNluRuntime;
use corebot_backend::core::nlu_engine::application::AnalyzeTextService;
use corebot_backend::core::restaurant::adapter::outbound::postgres_restaurant_repository::availability_repository::PostgresAvailabilityRepository;
use corebot_backend::core::restaurant::adapter::outbound::postgres_restaurant_repository::business_info_repository::PostgresBusinessInfoRepository;
use corebot_backend::core::restaurant::adapter::outbound::postgres_restaurant_repository::menu_repository::PostgresMenuRepository;
use corebot_backend::core::restaurant::adapter::outbound::postgres_restaurant_repository::reservation_repository::PostgresReservationRepository;
use corebot_backend::core::restaurant::application::database_restaurant_service::DatabaseRestaurantService;
use sqlx::PgPool;
use uuid::Uuid;

const BIND_ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    log_environment(load_environment());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let pool = PgPool::connect(&database_url())
        .await
        .unwrap_or_else(|error| panic!("Failed to connect to PostgreSQL: {error}"));
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap_or_else(|error| panic!("Failed to run database migrations: {error}"));

    let business_info_repository = PostgresBusinessInfoRepository::new(pool.clone());
    let menu_repository = PostgresMenuRepository::new(pool.clone());
    let reservation_repository = PostgresReservationRepository::new(pool.clone());
    let availability_repository = PostgresAvailabilityRepository::new(pool);
    let restaurant = Arc::new(DatabaseRestaurantService::new(
        default_business_id(),
        "en",
        business_info_repository,
        menu_repository,
        reservation_repository,
        availability_repository,
    ));
    let business_info_gateway =
        Arc::new(RestaurantBusinessInfoGateway::new(Arc::clone(&restaurant)));
    let menu_gateway = Arc::new(RestaurantMenuGateway::new(Arc::clone(&restaurant)));
    let reservation_gateway = Arc::new(RestaurantReservationGateway::new(Arc::clone(&restaurant)));
    let processor = ConversationProcessor::new();

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
        business_info_gateway.clone(),
        menu_gateway.clone(),
        menu_gateway.clone(),
        menu_gateway.clone(),
        menu_gateway,
        business_info_gateway.clone(),
        business_info_gateway.clone(),
        business_info_gateway.clone(),
        business_info_gateway.clone(),
        business_info_gateway.clone(),
        business_info_gateway.clone(),
        business_info_gateway.clone(),
        business_info_gateway,
        reservation_gateway,
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
    println!(
        "[startup] DATABASE_URL={}",
        std::env::var("DATABASE_URL")
            .map(|_| "<set>".to_string())
            .unwrap_or_else(|_| "<unset>".to_string())
    );
    println!(
        "[startup] COREBOT_DEFAULT_BUSINESS_ID={}",
        std::env::var("COREBOT_DEFAULT_BUSINESS_ID").unwrap_or_else(|_| "<unset>".to_string())
    );
}

fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| panic!("DATABASE_URL must be set for PostgreSQL persistence"))
}

fn default_business_id() -> Uuid {
    std::env::var("COREBOT_DEFAULT_BUSINESS_ID")
        .ok()
        .and_then(|value| Uuid::parse_str(&value).ok())
        .unwrap_or_else(|| {
            Uuid::parse_str("11111111-1111-1111-1111-111111111111")
                .expect("default business id literal must be a valid UUID")
        })
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
