mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use common::*;
use my_finances_api::security::headers::SECURITY_HEADERS;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn public_routes(pool: PgPool) {
    let app = app(pool);
    let res = send(&app, Method::GET, "/", None, None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body, json!("Index"));

    let res = send(&app, Method::GET, "/status", None, None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body, json!("Ok"));
}

#[sqlx::test]
async fn unknown_routes_and_methods_are_404(pool: PgPool) {
    let app = app(pool);
    for (method, path) in [
        (Method::GET, "/does-not-exist"),
        (Method::PATCH, "/year"),
        (Method::GET, "/budget/reorder"),
        (Method::POST, "/month/1"),
    ] {
        let res = send(&app, method, path, None, None).await;
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{path}");
        assert_eq!(res.body, json!({ "message": "Not Found" }));
    }
}

#[sqlx::test]
async fn security_headers_are_set(pool: PgPool) {
    let app = app(pool);
    for path in ["/", "/nope", "/year"] {
        let res = send(&app, Method::GET, path, None, None).await;
        for (name, value) in SECURITY_HEADERS {
            assert_eq!(res.headers.get(*name).unwrap(), value, "{name} on {path}");
        }
        assert_eq!(res.headers.get(header::CACHE_CONTROL).unwrap(), "no-store");
        assert!(res.headers.get("x-powered-by").is_none());
    }
}

fn preflight(origin: &str) -> Request<Body> {
    Request::builder()
        .method(Method::OPTIONS)
        .uri("/budget")
        .header(header::ORIGIN, origin)
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "PUT")
        .header(
            header::ACCESS_CONTROL_REQUEST_HEADERS,
            "authorization,content-type",
        )
        .body(Body::empty())
        .unwrap()
}

#[sqlx::test]
async fn cors_allowlist_outside_production(pool: PgPool) {
    let app = app(pool);
    for origin in [
        "https://my-finances-web.gustavoisensee.com",
        "http://localhost:3000",
        "http://localhost:3002",
    ] {
        let res = send_request(&app, preflight(origin)).await;
        assert!(res.status.is_success());
        assert_eq!(
            res.headers
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap(),
            origin
        );
        assert_eq!(
            res.headers
                .get(header::ACCESS_CONTROL_ALLOW_HEADERS)
                .unwrap(),
            "authorization,content-type"
        );
    }

    let res = send_request(&app, preflight("https://evil.example")).await;
    assert!(
        res.headers
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none()
    );
}

#[sqlx::test]
async fn cors_excludes_localhost_in_production(pool: PgPool) {
    let app = app_with_config(pool, test_config(true));
    for origin in ["http://localhost:3000", "http://localhost:3002"] {
        let res = send_request(&app, preflight(origin)).await;
        assert!(
            res.headers
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none()
        );
    }

    let origin = "https://my-finances-web.gustavoisensee.com";
    let res = send_request(&app, preflight(origin)).await;
    assert_eq!(
        res.headers
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap(),
        origin
    );
}

#[sqlx::test]
async fn request_body_validation(pool: PgPool) {
    let app = app(pool);
    let token = token("body-user");

    let too_large = json!({ "name": "x".repeat(1024 * 1024 + 1) });
    let res = send(
        &app,
        Method::POST,
        "/category",
        Some(&token),
        Some(too_large),
    )
    .await;
    assert_eq!(res.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(res.body, json!({ "message": "Request body is too large." }));

    let request = Request::builder()
        .method(Method::POST)
        .uri("/category")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{not json"))
        .unwrap();
    let res = send_request(&app, request).await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert!(res.body["message"].is_string());
}

#[sqlx::test]
async fn non_numeric_ids_are_400(pool: PgPool) {
    let app = app(pool);
    let token = token("id-user");
    for (method, path) in [
        (Method::GET, "/month/abc"),
        (Method::DELETE, "/budget/1.5"),
        (Method::DELETE, "/expense/99999999999"),
    ] {
        let res = send(&app, method, path, Some(&token), None).await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "{path}");
        assert_eq!(res.body, json!({ "message": "Invalid id." }));
    }
}

#[sqlx::test]
async fn internal_errors_are_hidden_in_production(pool: PgPool) {
    let token = token("error-user");
    let dev_app = app(pool.clone());
    let prod_app = app_with_config(pool.clone(), test_config(true));
    pool.close().await;

    let res = send(&prod_app, Method::GET, "/year", Some(&token), None).await;
    assert_eq!(res.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(res.body, json!({ "message": "Internal server error" }));

    let res = send(&dev_app, Method::GET, "/year", Some(&token), None).await;
    assert_eq!(res.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_ne!(res.body, json!({ "message": "Internal server error" }));
}
