use axum::{
    Router,
    extract::{Form, State},
    response::Html,
    routing::{get, post},
};
use moka::future::Cache;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;

static CACHE: Lazy<Cache<String, serde_json::Value>> = Lazy::new(|| {
    Cache::builder()
        .max_capacity(10_000)
        .time_to_live(std::time::Duration::from_secs(60 * 60)) // 1 hour
        .build()
});

#[derive(Deserialize)]
pub struct LoincInput {
    pub code: String,
}

#[derive(Clone)]
pub struct AppState {
    pub api_base_url: String,
    pub client: Client,
}

pub async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}

pub async fn validate_handler(
    State(state): State<AppState>,
    Form(input): Form<LoincInput>,
) -> Html<String> {
    let code = input.code.trim();

    if code.is_empty() {
        return Html(error("Please enter a valid LOINC code input."));
    }

    let url = format!(
        "{}/api/loinc_items/v3/search?sf=LOINC_NUM&df=LOINC_NUM,text&terms={}",
        state.api_base_url, code
    );

    // Try cache first
    if let Some(cached) = CACHE.get(&code.to_string()).await {
        return process_loinc_response(code, cached);
    }

    // Otherwise hit NIH using the shared state client
    let response = match state.client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return Html(error("Network error contacting NLM API.")),
    };

    let data: serde_json::Value = match response.json().await {
        Ok(v) => v,
        Err(_) => return Html(error("Failed to parse API response.")),
    };

    // Store in cache
    CACHE.insert(code.to_string(), data.clone()).await;

    process_loinc_response(code, data)
}

fn process_loinc_response(code: &str, data: serde_json::Value) -> Html<String> {
    let count = data.get(0).and_then(|v| v.as_i64()).unwrap_or(0);

    if count == 0 {
        return Html(invalid(code));
    }

    let items = match data.get(3).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Html(invalid(code)),
    };

    let first = match items.first().and_then(|v| v.as_array()) {
        Some(f) => f,
        None => return Html(invalid(code)),
    };

    let loinc_num = first.first().and_then(|v| v.as_str()).unwrap_or(code);
    let loinc_name = first
        .get(1)
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Description");

    Html(valid(loinc_num, loinc_name))
}

pub fn error(msg: &str) -> String {
    format!("<p class='text-red-600 font-medium'>{}</p>", msg)
}

pub fn invalid(code: &str) -> String {
    format!(
        "<div class='p-3 bg-red-50 border border-red-200 rounded-md text-red-800'>
            <p class='font-bold'>Invalid Code</p>
            <p class='text-sm mt-1'>Code '{}' was not found in the NLM database.</p>
        </div>",
        code
    )
}

pub fn valid(num: &str, name: &str) -> String {
    format!(
        "<div class='p-3 bg-green-50 border border-green-200 rounded-md text-green-800'>
            <p class='font-bold'>Valid LOINC Code: {}</p>
            <p class='text-sm mt-1'>Description: {}</p>
        </div>",
        num, name
    )
}

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
