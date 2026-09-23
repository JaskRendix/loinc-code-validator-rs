use crate::errors::LoincError;
use axum::{
    extract::{Form, State},
    response::Html,
};
use moka::future::Cache;
use once_cell::sync::Lazy;
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
    pub client: reqwest::Client,
}

pub async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}

pub async fn validate_handler(
    State(state): State<AppState>,
    Form(input): Form<LoincInput>,
) -> Result<Html<String>, LoincError> {
    let code = input.code.trim();

    if code.is_empty() {
        return Err(LoincError::EmptyInput);
    }

    let url = format!(
        "{}/api/loinc_items/v3/search?sf=LOINC_NUM&df=LOINC_NUM,text&terms={}",
        state.api_base_url, code
    );

    // Try cache first
    if let Some(cached) = CACHE.get(&code.to_string()).await {
        return process_loinc_response(code, cached);
    }

    // Otherwise hit NIH
    let response = state.client.get(&url).send().await?;
    let data: serde_json::Value = response.json().await?;

    // Store in cache
    CACHE.insert(code.to_string(), data.clone()).await;

    process_loinc_response(code, data)
}

fn process_loinc_response(code: &str, data: serde_json::Value) -> Result<Html<String>, LoincError> {
    let count = data.get(0).and_then(|v| v.as_i64()).unwrap_or(0);

    if count == 0 {
        return Err(LoincError::NotFound(code.to_string()));
    }

    let items = data
        .get(3)
        .and_then(|v| v.as_array())
        .ok_or_else(|| LoincError::NotFound(code.to_string()))?;

    let first = items
        .first()
        .and_then(|v| v.as_array())
        .ok_or_else(|| LoincError::NotFound(code.to_string()))?;

    let loinc_num = first.first().and_then(|v| v.as_str()).unwrap_or(code);
    let loinc_name = first
        .get(1)
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Description");

    Ok(Html(valid(loinc_num, loinc_name)))
}

fn valid(num: &str, name: &str) -> String {
    include_str!("../templates/valid.html")
        .replace("{num}", num)
        .replace("{name}", name)
}
