use axum::Router;
use corebot_backend::core::conversation::adapter::input::web::routes::conversation_routes;

const BIND_ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    let app = Router::new().merge(conversation_routes());

    let listener = tokio::net::TcpListener::bind(BIND_ADDRESS)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {BIND_ADDRESS}: {e}"));

    println!("Server listening on {BIND_ADDRESS}");

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("Server error: {e}"));
}
