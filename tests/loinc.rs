use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
};
use loinc_code_validator_rs::{index_handler, validate_handler};
use tower::Service;

#[tokio::test]
async fn test_valid_codes() {
    let mut app = Router::new()
        .route("/", get(index_handler))
        .route("/validate", post(validate_handler));

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
    let mut app = Router::new()
        .route("/", get(index_handler))
        .route("/validate", post(validate_handler));

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
    let mut app = Router::new()
        .route("/", get(index_handler))
        .route("/validate", post(validate_handler));

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
    let mut app = Router::new()
        .route("/", get(index_handler))
        .route("/validate", post(validate_handler));

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
    let mut app = Router::new()
        .route("/", get(index_handler))
        .route("/validate", post(validate_handler));

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
