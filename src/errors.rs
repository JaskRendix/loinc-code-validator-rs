use axum::{
    response::Response,
    response::{Html, IntoResponse},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoincError {
    #[error("Please enter a valid LOINC code input.")]
    EmptyInput,

    #[error("Invalid LOINC format. Expected formats like '4544-3' or '10154-3'.")]
    InvalidFormat,

    #[error("Network error contacting NLM API.")]
    Network(#[from] reqwest::Error),

    #[error("Failed to parse API response.")]
    Parse(#[from] serde_json::Error),

    #[error("Code '{0}' was not found in the NLM database.")]
    NotFound(String),
}

impl IntoResponse for LoincError {
    fn into_response(self) -> Response {
        let html_content = match self {
            LoincError::EmptyInput => include_str!("../templates/error_empty.html")
                .replace("{message}", &self.to_string()),

            LoincError::InvalidFormat => include_str!("../templates/error_empty.html")
                .replace("{message}", &self.to_string()),

            LoincError::NotFound(ref code) => {
                include_str!("../templates/error_not_found.html").replace("{code}", code)
            }
            _ => include_str!("../templates/error_generic.html")
                .replace("{message}", &self.to_string()),
        };
        Html(html_content).into_response()
    }
}
