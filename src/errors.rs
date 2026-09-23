use axum::{
    response::Response,
    response::{Html, IntoResponse},
};
use thiserror::Error;

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
    fn into_response(self) -> Response {
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
