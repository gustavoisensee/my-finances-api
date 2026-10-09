#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use chrono::Utc;
use http_body_util::BodyExt;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use my_finances_api::auth::firebase::FirebaseVerifier;
use my_finances_api::config::Config;
use my_finances_api::{AppState, build_app};
use rsa::RsaPrivateKey;
use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

pub const PROJECT_ID: &str = "test-project";
pub const KID: &str = "test-kid";
pub const ALLOWLISTED_ADMIN: &str = "allowlisted-admin";

pub struct KeyPair {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
    pub public_pem: String,
}

fn generate_key_pair() -> KeyPair {
    let private = RsaPrivateKey::new(&mut rand::thread_rng(), 2048).expect("generate RSA key");
    let private_pem = private
        .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
        .expect("encode private key");
    let public_pem = private
        .to_public_key()
        .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
        .expect("encode public key");
    KeyPair {
        encoding: EncodingKey::from_rsa_pem(private_pem.as_bytes()).unwrap(),
        decoding: DecodingKey::from_rsa_pem(public_pem.as_bytes()).unwrap(),
        public_pem,
    }
}

/// The key the test verifier trusts.
pub fn signing_key() -> &'static KeyPair {
    static KEY: OnceLock<KeyPair> = OnceLock::new();
    KEY.get_or_init(generate_key_pair)
}

/// A valid RSA key the verifier does not trust.
pub fn attacker_key() -> &'static KeyPair {
    static KEY: OnceLock<KeyPair> = OnceLock::new();
    KEY.get_or_init(generate_key_pair)
}

pub fn test_config(is_production: bool) -> Config {
    Config {
        database_url: String::new(),
        firebase_project_id: PROJECT_ID.to_owned(),
        admin_uids: HashSet::from([ALLOWLISTED_ADMIN.to_owned()]),
        is_production,
    }
}

pub fn app(pool: PgPool) -> Router {
    app_with_config(pool, test_config(false))
}

pub fn app_with_config(pool: PgPool, config: Config) -> Router {
    let verifier = FirebaseVerifier::with_static_keys(
        PROJECT_ID,
        HashMap::from([(KID.to_owned(), signing_key().decoding.clone())]),
    );
    build_app(AppState::new(config, pool, verifier))
}

pub fn valid_claims(uid: &str) -> Value {
    let now = Utc::now().timestamp();
    json!({
        "iss": format!("https://securetoken.google.com/{PROJECT_ID}"),
        "aud": PROJECT_ID,
        "sub": uid,
        "user_id": uid,
        "iat": now - 10,
        "exp": now + 3600,
        "auth_time": now - 60,
    })
}

pub fn sign(claims: &Value, kid: Option<&str>, key: &EncodingKey) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = kid.map(str::to_owned);
    jsonwebtoken::encode(&header, claims, key).expect("sign token")
}

/// A valid Firebase ID token for `uid`.
pub fn token(uid: &str) -> String {
    sign(&valid_claims(uid), Some(KID), &signing_key().encoding)
}

/// A valid token with extra claims merged in (e.g. custom admin claims).
pub fn token_with(uid: &str, extra: Value) -> String {
    let mut claims = valid_claims(uid);
    for (k, v) in extra.as_object().unwrap() {
        claims[k] = v.clone();
    }
    sign(&claims, Some(KID), &signing_key().encoding)
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Value,
}

pub async fn send(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> TestResponse {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let request = match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string())),
        None => builder.body(Body::empty()),
    }
    .unwrap();
    send_request(app, request).await
}

pub async fn send_request(app: &Router, request: Request<Body>) -> TestResponse {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    TestResponse {
        status,
        headers,
        body,
    }
}

/// Convenience wrapper bound to one user's token.
pub struct Client {
    pub app: Router,
    pub token: String,
}

impl Client {
    pub fn new(app: &Router, uid: &str) -> Self {
        Self {
            app: app.clone(),
            token: token(uid),
        }
    }

    pub fn admin(app: &Router) -> Self {
        Self {
            app: app.clone(),
            token: token_with("admin-uid", json!({ "admin": true })),
        }
    }

    pub async fn get(&self, path: &str) -> TestResponse {
        send(&self.app, Method::GET, path, Some(&self.token), None).await
    }

    pub async fn post(&self, path: &str, body: Value) -> TestResponse {
        send(&self.app, Method::POST, path, Some(&self.token), Some(body)).await
    }

    pub async fn put(&self, path: &str, body: Value) -> TestResponse {
        send(&self.app, Method::PUT, path, Some(&self.token), Some(body)).await
    }

    pub async fn delete(&self, path: &str) -> TestResponse {
        send(&self.app, Method::DELETE, path, Some(&self.token), None).await
    }
}

pub struct Seed {
    pub year_2025: i32,
    pub year_2026: i32,
    pub global_category: i32,
}

pub async fn seed(pool: &PgPool) -> Seed {
    let year_2025: i32 = sqlx::query_scalar(
        r#"INSERT INTO "Year" (value, "createdAt") VALUES (2025, now()) RETURNING id"#,
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let year_2026: i32 = sqlx::query_scalar(
        r#"INSERT INTO "Year" (value, "createdAt") VALUES (2026, now()) RETURNING id"#,
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let global_category: i32 = sqlx::query_scalar(
        r#"INSERT INTO "Category" (name, "userId", "createdAt") VALUES ('Housing', NULL, now()) RETURNING id"#,
    )
    .fetch_one(pool)
    .await
    .unwrap();
    Seed {
        year_2025,
        year_2026,
        global_category,
    }
}

pub fn id(value: &Value) -> i64 {
    value["id"].as_i64().expect("response has an id")
}

pub async fn create_month(client: &Client, year_id: i32, value: i32) -> i64 {
    let res = client
        .post(
            "/month",
            json!({
                "value": value,
                "description": format!("Month {value}"),
                "createdAt": "2025-01-01T00:00:00.000Z",
                "yearId": year_id,
            }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{:?}", res.body);
    id(&res.body)
}

pub async fn create_budget(client: &Client, month_id: i64, description: &str) -> Value {
    let res = client
        .post(
            "/budget",
            json!({
                "description": description,
                "value": 100.0,
                "createdAt": "2025-01-01T00:00:00.000Z",
                "monthId": month_id,
            }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{:?}", res.body);
    res.body
}

pub async fn create_income(client: &Client, month_id: i64, description: &str) -> Value {
    let res = client
        .post(
            "/income",
            json!({
                "description": description,
                "value": 1000.0,
                "createdAt": "2025-01-01T00:00:00.000Z",
                "monthId": month_id,
            }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{:?}", res.body);
    res.body
}

pub async fn create_expense(client: &Client, budget_id: i64, category_id: i64) -> Value {
    let res = client
        .post(
            "/expense",
            json!({
                "description": "Supermarket",
                "value": 85.5,
                "createdAt": "2025-01-15T10:30:00.000Z",
                "budgetId": budget_id,
                "categoryId": category_id,
            }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{:?}", res.body);
    res.body
}
