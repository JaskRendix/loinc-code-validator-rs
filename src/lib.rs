pub mod errors;
pub mod handlers;

use axum::{
    Router,
    routing::{get, post},
};
use handlers::{AppState, index_handler, validate_handler};
use reqwest::Client;

pub fn app() -> Router {
    let state = AppState {
        api_base_url: std::env::var("NIH_API_BASE")
            .unwrap_or_else(|_| "https://clinicaltables.nlm.nih.gov".to_string()),
        client: Client::new(),
    };

    Router::new()
        .route("/", get(index_handler))
        .route("/validate", post(validate_handler))
        .with_state(state)
}
