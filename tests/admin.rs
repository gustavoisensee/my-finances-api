mod common;

use axum::http::StatusCode;
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

#[sqlx::test]
async fn user_admin_crud(pool: PgPool) {
    let admin = Client::admin(&app(pool));

    let res = admin.post("/user", json!({})).await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body, json!({ "message": "firebaseUid is required." }));

    let res = admin
        .post("/user", json!({ "firebaseUid": "new-firebase-uid" }))
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    assert_eq!(res.body["firebaseUid"], "new-firebase-uid");
    assert_eq!(res.body["clerkId"], Value::Null);
    let user_id = id(&res.body);

    let res = admin
        .post("/user", json!({ "firebaseUid": "new-firebase-uid" }))
        .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        res.body,
        json!({ "message": "firebaseUid is already in use." })
    );

    let res = admin.get("/user").await;
    assert_eq!(res.status, StatusCode::OK);
    let users = res.body.as_array().unwrap();
    assert!(users.iter().any(|u| u["id"] == user_id));
    assert_eq!(
        users[0].as_object().unwrap().keys().collect::<Vec<_>>(),
        vec!["clerkId", "createdAt", "firebaseUid", "id"]
    );

    let res = admin
        .put(
            &format!("/user/{user_id}"),
            json!({ "firebaseUid": "renamed-uid" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["firebaseUid"], "renamed-uid");

    let res = admin
        .put("/user/99999", json!({ "firebaseUid": "x" }))
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body, json!({ "message": "User not found." }));

    assert_eq!(
        admin.delete("/user/99999").await.status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test]
async fn deleting_a_user_removes_all_their_data(pool: PgPool) {
    let seed = seed(&pool).await;
    let app = app(pool.clone());
    let admin = Client::admin(&app);
    let alice = Client::new(&app, "alice");
    let bob = Client::new(&app, "bob");

    let month_id = create_month(&alice, seed.year_2025, 1).await;
    let budget = id(&create_budget(&alice, month_id, "Groceries").await);
    create_income(&alice, month_id, "Salary").await;
    let category = id(&alice
        .post("/category", json!({ "name": "Alice only" }))
        .await
        .body);
    create_expense(&alice, budget, category).await;
    create_expense(&alice, budget, seed.global_category as i64).await;

    let bobs_month = create_month(&bob, seed.year_2025, 1).await;

    let alice_id: i32 =
        sqlx::query_scalar(r#"SELECT id FROM "User" WHERE "firebaseUid" = 'alice'"#)
            .fetch_one(&pool)
            .await
            .unwrap();

    let res = admin.delete(&format!("/user/{alice_id}")).await;
    assert_eq!(res.status, StatusCode::OK, "{:?}", res.body);
    assert_eq!(res.body["firebaseUid"], "alice");

    for (table, expected) in [
        ("User", 1),
        ("Month", 1),
        ("Budget", 0),
        ("Income", 0),
        ("Expense", 0),
        ("Category", 1),
    ] {
        let count: i64 = sqlx::query_scalar(&format!(r#"SELECT COUNT(*) FROM "{table}""#))
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, expected, "{table}");
    }

    assert_eq!(
        bob.get(&format!("/month/{bobs_month}")).await.status,
        StatusCode::OK
    );
}

#[sqlx::test]
async fn access_tokens_list(pool: PgPool) {
    sqlx::query(
        r#"INSERT INTO "AccessToken" (name, value, "expirationDate", "createdAt")
           VALUES ('ci', 'opaque', '2030-01-01T00:00:00', '2025-01-01T00:00:00')"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let admin = Client::admin(&app(pool));
    let res = admin.get("/access-token").await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(
        res.body,
        json!([{
            "id": res.body[0]["id"],
            "name": "ci",
            "value": "opaque",
            "expirationDate": "2030-01-01T00:00:00.000Z",
            "createdAt": "2025-01-01T00:00:00.000Z",
        }])
    );
}
