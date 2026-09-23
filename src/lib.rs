use axum::{
    Router,
    extract::Form,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
pub struct LoincInput {
    pub code: String,
}

#[derive(Error, Debug)]
pub enum LoincError {
    #[error("Please enter a valid LOINC code input.")]
    EmptyInput,

    #[error("Network error contacting NLM API.")]
    Network(#[from] reqwest::Error),

    #[error("Failed to parse API response.")]
    Parse(#[from] serde_json::Error),

    #[error("Code '{0}' was not found in the NLM database.")]
    NotFound(String),
}

impl IntoResponse for LoincError {
    fn into_response(self) -> axum::response::Response {
        let html_content = match self {
            LoincError::EmptyInput => {
                format!("<p class='text-red-600 font-medium'>{}</p>", self)
            }
            LoincError::NotFound(ref code) => {
                format!(
                    "<div class='p-3 bg-red-50 border border-red-200 rounded-md text-red-800'>
                        <p class='font-bold'>Invalid Code</p>
                        <p class='text-sm mt-1'>Code '{}' was not found in the NLM database.</p>
                    </div>",
                    code
                )
            }
            _ => {
                format!(
                    "<div class='p-3 bg-red-50 border border-red-200 rounded-md text-red-800'>
                        <p class='font-bold'>Error</p>
                        <p class='text-sm mt-1'>{}</p>
                    </div>",
                    self
                )
            }
        };
        Html(html_content).into_response()
    }
}

pub async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}

pub async fn validate_handler(Form(input): Form<LoincInput>) -> Result<Html<String>, LoincError> {
    let code = input.code.trim();

    if code.is_empty() {
        return Err(LoincError::EmptyInput);
    }

    let url = format!(
        "https://clinicaltables.nlm.nih.gov/api/loinc_items/v3/search?sf=LOINC_NUM&df=LOINC_NUM,text&terms={}",
        code
    );

    let client = Client::new();
    let response = client.get(&url).send().await?;

    let data: serde_json::Value = response.json().await?;

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
    Router::new()
        .route("/", get(index_handler))
        .route("/validate", post(validate_handler))
}
