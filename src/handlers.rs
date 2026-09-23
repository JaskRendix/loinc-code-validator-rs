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
pub struct NlmLoincResponse(
    pub usize,                     // count
    pub Vec<String>,               // codes
    pub Option<serde_json::Value>, // extra metadata
    pub Vec<(String, String)>,     // pairs of (LOINC_NUM, Description)
);

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

fn is_valid_loinc_format(code: &str) -> bool {
    let parts: Vec<&str> = code.split('-').collect();
    if parts.len() != 2 {
        return false;
    }
    let num_part = parts[0];
    let check_part = parts[1];

    // LOINC rules: 1 to 5 digits before the hyphen, exactly 1 digit after
    !num_part.is_empty()
        && num_part.len() <= 5
        && num_part.chars().all(|c| c.is_ascii_digit())
        && check_part.len() == 1
        && check_part.chars().all(|c| c.is_ascii_digit())
}

pub async fn validate_handler(
    State(state): State<AppState>,
    Form(input): Form<LoincInput>,
) -> Result<Html<String>, LoincError> {
    let code = input.code.trim();

    if code.is_empty() {
        return Err(LoincError::EmptyInput);
    }

    if !is_valid_loinc_format(code) {
        return Err(LoincError::InvalidFormat);
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
    let response: NlmLoincResponse = serde_json::from_value(data)?;

    if response.0 == 0 {
        return Err(LoincError::NotFound(code.to_string()));
    }

    if let Some((num, name)) = response.3.first() {
        Ok(Html(valid(num, name)))
    } else {
        Err(LoincError::NotFound(code.to_string()))
    }
}

fn valid(num: &str, name: &str) -> String {
    include_str!("../templates/valid.html")
        .replace("{num}", num)
        .replace("{name}", name)
}
