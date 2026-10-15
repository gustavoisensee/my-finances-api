mod common;

use axum::http::{Method, StatusCode};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use common::*;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn status_for(pool: &PgPool, token: &str) -> (StatusCode, Value) {
    let app = app(pool.clone());
    let res = send(&app, Method::GET, "/year", Some(token), None).await;
    (res.status, res.body)
}

fn assert_unauthenticated(status: StatusCode, body: &Value) {
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        body,
        &json!({
            "error": "Unauthorized",
            "message": "Authentication required. Please provide a valid Firebase ID token.",
            "code": "UNAUTHENTICATED",
        })
    );
}

fn claims_with(uid: &str, overrides: Value) -> Value {
    let mut claims = valid_claims(uid);
    for (k, v) in overrides.as_object().unwrap() {
        if v.is_null() {
            claims.as_object_mut().unwrap().remove(k);
        } else {
            claims[k] = v.clone();
        }
    }
    claims
}

#[sqlx::test]
async fn missing_or_malformed_authorization_header(pool: PgPool) {
    let app = app(pool);
    let res = send(&app, Method::GET, "/year", None, None).await;
    assert_unauthenticated(res.status, &res.body);

    for header in ["Basic abc", "Bearer ", "Bearer    ", "bearer token"] {
        let request = axum::http::Request::builder()
            .uri("/year")
            .header("authorization", header)
            .body(axum::body::Body::empty())
            .unwrap();
        let res = send_request(&app, request).await;
        assert_unauthenticated(res.status, &res.body);
    }
}

#[sqlx::test]
async fn valid_token_is_accepted_and_creates_the_user_once(pool: PgPool) {
    let token = token("firebase-user-1");
    let (status, _) = status_for(&pool, &token).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = status_for(&pool, &token).await;
    assert_eq!(status, StatusCode::OK);

    let count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM "User" WHERE "firebaseUid" = 'firebase-user-1'"#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
}

#[sqlx::test]
async fn rejects_tokens_with_invalid_claims(pool: PgPool) {
    let now = Utc::now().timestamp();
    let cases = [
        ("wrong audience", json!({ "aud": "other-project" })),
        (
            "wrong issuer",
            json!({ "iss": "https://securetoken.google.com/other-project" }),
        ),
        ("expired", json!({ "exp": now - 120 })),
        ("issued in the future", json!({ "iat": now + 600 })),
        ("auth_time in the future", json!({ "auth_time": now + 600 })),
        ("auth_time missing", json!({ "auth_time": null })),
        ("empty subject", json!({ "sub": "" })),
        ("subject too long", json!({ "sub": "x".repeat(129) })),
    ];

    for (name, overrides) in cases {
        let token = sign(
            &claims_with("uid", overrides),
            Some(KID),
            &signing_key().encoding,
        );
        let (status, body) = status_for(&pool, &token).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "case: {name}");
        assert_unauthenticated(status, &body);
    }
}

#[sqlx::test]
async fn rejects_tokens_with_bad_keys_or_signatures(pool: PgPool) {
    let claims = valid_claims("uid");

    let unknown_kid = sign(&claims, Some("other-kid"), &signing_key().encoding);
    let missing_kid = sign(&claims, None, &signing_key().encoding);
    let wrong_key = sign(&claims, Some(KID), &attacker_key().encoding);

    for token in [unknown_kid, missing_kid, wrong_key] {
        let (status, body) = status_for(&pool, &token).await;
        assert_unauthenticated(status, &body);
    }
}

#[sqlx::test]
async fn rejects_algorithm_confusion_and_unsigned_tokens(pool: PgPool) {
    let claims = valid_claims("uid");

    // HS256 signed with the public key as the HMAC secret.
    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some(KID.to_owned());
    let hs256 = jsonwebtoken::encode(
        &header,
        &claims,
        &EncodingKey::from_secret(signing_key().public_pem.as_bytes()),
    )
    .unwrap();

    let unsigned = format!(
        "{}.{}.",
        URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"JWT","kid":"test-kid"}"#),
        URL_SAFE_NO_PAD.encode(claims.to_string()),
    );

    for token in [hs256, unsigned, "not-a-jwt".to_owned()] {
        let (status, body) = status_for(&pool, &token).await;
        assert_unauthenticated(status, &body);
    }
}

#[sqlx::test]
async fn admin_routes_require_admin(pool: PgPool) {
    let app = app(pool);

    let res = send(&app, Method::GET, "/user", None, None).await;
    assert_unauthenticated(res.status, &res.body);

    let forbidden = json!({
        "error": "Forbidden",
        "message": "Admin access required.",
        "code": "ADMIN_ACCESS_REQUIRED",
    });
    for token in [
        token("regular-user"),
        token_with("regular-user", json!({ "admin": "true" })),
        token_with("regular-user", json!({ "role": "user" })),
    ] {
        for path in ["/user", "/access-token"] {
            let res = send(&app, Method::GET, path, Some(&token), None).await;
            assert_eq!(res.status, StatusCode::FORBIDDEN, "{path}");
            assert_eq!(res.body, forbidden);
        }
    }

    for token in [
        token_with("claim-admin", json!({ "admin": true })),
        token_with("role-admin", json!({ "role": "admin" })),
        token(ALLOWLISTED_ADMIN),
    ] {
        for path in ["/user", "/access-token"] {
            let res = send(&app, Method::GET, path, Some(&token), None).await;
            assert_eq!(res.status, StatusCode::OK, "{path}");
        }
    }
}
