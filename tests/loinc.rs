use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
};
use loinc_code_validator_rs::{AppState, index_handler, validate_handler};
use tower::Service;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn build_test_app(api_base_url: String) -> Router {
    let state = AppState {
        api_base_url,
        client: reqwest::Client::new(),
    };
    Router::new()
        .route("/", get(index_handler))
        .route("/validate", post(validate_handler))
        .with_state(state)
}

#[tokio::test]
async fn test_mocked_valid_code() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/loinc_items/v3/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            1,
            ["718-7"],
            null,
            [["718-7", "Hemoglobin [Mass/volume] in Blood"]]
        ])))
        .mount(&server)
        .await;

    let mut app = build_test_app(server.uri());

    let req = Request::builder()
        .method("POST")
        .uri("/validate")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("code=718-7"))
        .unwrap();

    let res = app.call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();

    let html = String::from_utf8_lossy(&bytes);
    assert!(html.contains("Valid LOINC Code"));
}

#[tokio::test]
async fn test_valid_codes() {
    let mut app = build_test_app("https://clinicaltables.nlm.nih.gov".to_string());

    let codes = vec!["718-7", "2951-2", "4548-4", "6690-2"];

    for code in codes {
        let body = format!("code={}", code);

        let req = Request::builder()
            .method("POST")
            .uri("/validate")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap();

        let res = app.call(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();

        let html = String::from_utf8_lossy(&bytes);
        assert!(html.contains("Valid LOINC Code"));
    }
}

#[tokio::test]
async fn test_invalid_codes() {
    let mut app = build_test_app("https://clinicaltables.nlm.nih.gov".to_string());

    let codes = vec!["999999-9", "NOT-A-CODE", "abc-xyz"];

    for code in codes {
        let body = format!("code={}", code);

        let req = Request::builder()
            .method("POST")
            .uri("/validate")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap();

        let res = app.call(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();

        let html = String::from_utf8_lossy(&bytes);
        assert!(html.contains("Invalid Code"));
    }
}

#[tokio::test]
async fn test_empty_input() {
    let mut app = build_test_app("http://localhost:0".to_string());

    let req = Request::builder()
        .method("POST")
        .uri("/validate")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("code="))
        .unwrap();

    let res = app.call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();

    let html = String::from_utf8_lossy(&bytes);
    assert!(html.contains("Please enter a valid LOINC code input."));
}

#[tokio::test]
async fn test_malformed_form_body() {
    let mut app = build_test_app("http://localhost:0".to_string());

    let req = Request::builder()
        .method("POST")
        .uri("/validate")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("not_code_field=123"))
        .unwrap();

    let res = app.call(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_whitespace_input() {
    let mut app = build_test_app("http://localhost:0".to_string());

    let req = Request::builder()
        .method("POST")
        .uri("/validate")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("code=   "))
        .unwrap();

    let res = app.call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();

    let html = String::from_utf8_lossy(&bytes);
    assert!(html.contains("Please enter a valid LOINC code input."));
}
