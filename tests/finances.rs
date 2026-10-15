mod common;

use axum::http::StatusCode;
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

fn ids(body: &Value) -> Vec<i64> {
    body.as_array().unwrap().iter().map(id).collect()
}

fn indexes(body: &Value) -> Vec<i64> {
    body.as_array()
        .unwrap()
        .iter()
        .map(|v| v["index"].as_i64().unwrap())
        .collect()
}

#[sqlx::test]
async fn years_list_with_take(pool: PgPool) {
    seed(&pool).await;
    let alice = Client::new(&app(pool), "alice");

    let res = alice.get("/year").await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body.as_array().unwrap().len(), 2);
    let year = &res.body[0];
    assert_eq!(year["value"], 2025);
    let created_at = year["createdAt"].as_str().unwrap();
    assert!(
        created_at.ends_with('Z') && created_at.len() == 24,
        "{created_at}"
    );

    assert_eq!(
        alice
            .get("/year?take=1")
            .await
            .body
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        alice
            .get("/year?take=-3")
            .await
            .body
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        alice
            .get("/year?take=abc")
            .await
            .body
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[sqlx::test]
async fn month_crud_and_validation(pool: PgPool) {
    let seed = seed(&pool).await;
    let alice = Client::new(&app(pool), "alice");

    let res = alice
        .post(
            "/month",
            json!({ "value": 13, "description": "x", "createdAt": "2025-01-01T00:00:00Z", "yearId": seed.year_2025 }),
        )
        .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        res.body,
        json!({ "message": "Value must be a number between 1 and 12." })
    );

    let res = alice
        .post(
            "/month",
            json!({ "value": 1, "description": "x", "createdAt": "2025-01-01T00:00:00Z", "yearId": 9999 }),
        )
        .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body, json!({ "message": "Year does not exist." }));

    let res = alice
        .post(
            "/month",
            json!({ "value": 1, "description": "January", "createdAt": "2025-01-01T00:00:00.000Z", "yearId": seed.year_2025 }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    let month = res.body.clone();
    let month_id = id(&month);
    assert_eq!(month["description"], "January");
    assert_eq!(month["createdAt"], "2025-01-01T00:00:00.000Z");
    assert_eq!(month["yearId"], seed.year_2025);
    assert!(month.get("incomes").is_none());

    let res = alice.get("/month").await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body, json!({ "message": "yearId must be passed." }));

    let res = alice
        .get(&format!("/month?yearId={}", seed.year_2025))
        .await;
    assert_eq!(ids(&res.body), vec![month_id]);
    assert!(res.body[0].get("incomes").is_none());
    assert!(res.body[0].get("budgets").is_none());
    assert!(
        alice
            .get(&format!("/month?yearId={}", seed.year_2026))
            .await
            .body
            .as_array()
            .unwrap()
            .is_empty()
    );

    // Partial update keeps the description, like Prisma ignoring undefined.
    let res = alice
        .put(&format!("/month/{month_id}"), json!({ "value": 2 }))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["value"], 2);
    assert_eq!(res.body["description"], "January");

    let res = alice
        .put(
            &format!("/month/{month_id}"),
            json!({ "value": 3, "description": "March", "yearId": seed.year_2026 }),
        )
        .await;
    assert_eq!(res.body["description"], "March");
    assert_eq!(res.body["yearId"], seed.year_2026);

    let res = alice.put("/month/99999", json!({ "value": 3 })).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);

    let res = alice.get("/month/99999").await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body, json!({ "message": "Month not found." }));

    let res = alice.delete(&format!("/month/{month_id}")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(id(&res.body), month_id);
    assert_eq!(
        alice.delete(&format!("/month/{month_id}")).await.status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test]
async fn month_includes_are_conditional(pool: PgPool) {
    let seed = seed(&pool).await;
    let alice = Client::new(&app(pool), "alice");
    let month_id = create_month(&alice, seed.year_2025, 1).await;
    let budget = create_budget(&alice, month_id, "Groceries").await;
    create_income(&alice, month_id, "Salary").await;
    create_expense(&alice, id(&budget), seed.global_category as i64).await;

    let base = format!("/month/{month_id}");
    let res = alice.get(&format!("{base}?iIncomes=true")).await;
    assert_eq!(res.body["incomes"].as_array().unwrap().len(), 1);
    assert!(res.body.get("budgets").is_none());

    let res = alice.get(&format!("{base}?iBudgets=true")).await;
    assert!(res.body.get("incomes").is_none());
    assert_eq!(res.body["budgets"].as_array().unwrap().len(), 1);
    assert!(res.body["budgets"][0].get("expenses").is_none());

    let res = alice
        .get(&format!(
            "{base}?iIncomes=true&iBudgets=TRUE&iExpenses=true"
        ))
        .await;
    assert_eq!(res.body["incomes"][0]["description"], "Salary");
    assert_eq!(res.body["budgets"][0]["description"], "Groceries");
    assert_eq!(res.body["budgets"][0]["expenses"][0]["value"], 85.5);
    assert_eq!(
        res.body["budgets"][0]["expenses"][0]["createdAt"],
        "2025-01-15T10:30:00.000Z"
    );

    let res = alice
        .get(&format!(
            "/month?yearId={}&iIncomes=true&iBudgets=true&iExpenses=true",
            seed.year_2025
        ))
        .await;
    assert_eq!(
        res.body[0]["budgets"][0]["expenses"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[sqlx::test]
async fn month_list_with_take_returns_every_month_of_the_year(pool: PgPool) {
    let seed = seed(&pool).await;
    let alice = Client::new(&app(pool), "alice");

    // Same sequence as the dashboard: January to September, a copy of
    // September, then three copies of October. That is more than the default
    // page size of 10, so the dashboard asks for `take=100` to get all of them.
    let mut expected = Vec::new();
    for value in 1..=9 {
        expected.push(create_month(&alice, seed.year_2026, value).await);
    }
    let september = expected[8];
    let res = alice
        .post(
            &format!("/month/{september}/copy"),
            json!({ "value": 10, "yearId": seed.year_2026 }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    expected.push(id(&res.body));

    let october = expected[9];
    for _ in 0..3 {
        let res = alice
            .post(
                &format!("/month/{october}/copy"),
                json!({ "value": 11, "yearId": seed.year_2026 }),
            )
            .await;
        assert_eq!(res.status, StatusCode::CREATED);
        assert_eq!(res.body["description"], "Copy from October 2026");
        expected.push(id(&res.body));
    }

    let res = alice
        .get(&format!("/month?yearId={}&take=100", seed.year_2026))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(ids(&res.body), expected);
}

#[sqlx::test]
async fn month_copy(pool: PgPool) {
    let seed = seed(&pool).await;
    let alice = Client::new(&app(pool), "alice");
    let source = create_month(&alice, seed.year_2025, 3).await;
    let budget = create_budget(&alice, source, "Rent").await;
    create_budget(&alice, source, "Food").await;
    create_income(&alice, source, "Salary").await;
    create_expense(&alice, id(&budget), seed.global_category as i64).await;

    let res = alice
        .post(&format!("/month/{source}/copy"), json!({ "value": 4 }))
        .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body, json!({ "message": "yearId is required." }));

    let res = alice
        .post(
            "/month/99999/copy",
            json!({ "value": 4, "yearId": seed.year_2025 }),
        )
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body, json!({ "message": "Source month not found." }));

    let res = alice
        .post(
            &format!("/month/{source}/copy"),
            json!({ "value": 4, "yearId": seed.year_2026 }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    assert_eq!(res.body["description"], "Copy from March 2025");
    assert_eq!(res.body["value"], 4);
    assert_eq!(res.body["yearId"], seed.year_2026);
    let budgets = res.body["budgets"].as_array().unwrap();
    assert_eq!(budgets.len(), 2);
    assert_eq!(budgets[0]["description"], "Rent");
    assert_eq!(budgets[1]["index"], 1);
    assert!(budgets[0].get("expenses").is_none());
    assert_eq!(res.body["incomes"].as_array().unwrap().len(), 1);

    let copy_id = id(&res.body);
    let copied = alice
        .get(&format!("/month/{copy_id}?iBudgets=true&iExpenses=true"))
        .await;
    assert!(
        copied.body["budgets"][0]["expenses"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test]
async fn budget_lifecycle_and_ordering(pool: PgPool) {
    let seed = seed(&pool).await;
    let alice = Client::new(&app(pool), "alice");
    let month_id = create_month(&alice, seed.year_2025, 1).await;

    let res = alice
        .post("/budget", json!({ "description": "ab", "value": 10, "createdAt": "2025-01-01T00:00:00Z", "monthId": month_id }))
        .await;
    assert_eq!(
        res.body,
        json!({ "message": "Name must have at least 3 characters." })
    );
    let res = alice
        .post("/budget", json!({ "description": "abc", "value": 0, "createdAt": "2025-01-01T00:00:00Z", "monthId": month_id }))
        .await;
    assert_eq!(
        res.body,
        json!({ "message": "Value must be a number and greater than 0." })
    );
    let res = alice
        .post("/budget", json!({ "description": "abc", "value": 1, "createdAt": "2025-01-01T00:00:00Z", "monthId": 99999 }))
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body, json!({ "message": "Month does not exist." }));

    let a = create_budget(&alice, month_id, "Budget A").await;
    let b = create_budget(&alice, month_id, "Budget B").await;
    let c = create_budget(&alice, month_id, "Budget C").await;
    assert_eq!(
        (a["index"].clone(), b["index"].clone(), c["index"].clone()),
        (json!(0), json!(1), json!(2))
    );
    assert_eq!(a["color"], Value::Null);

    // Color is optional on update: omitted keeps the value, null clears it.
    let res = alice
        .put(&format!("/budget/{}", id(&a)), json!({ "description": "Budget A", "value": 50, "color": "#ff0000", "monthId": month_id }))
        .await;
    assert_eq!(res.body["color"], "#ff0000");
    let res = alice
        .put(
            &format!("/budget/{}", id(&a)),
            json!({ "description": "Budget A2", "value": 60, "monthId": month_id }),
        )
        .await;
    assert_eq!(res.body["color"], "#ff0000");
    assert_eq!(res.body["value"], 60.0);
    let res = alice
        .put(
            &format!("/budget/{}", id(&a)),
            json!({ "description": "Budget A2", "value": 60, "color": null, "monthId": month_id }),
        )
        .await;
    assert_eq!(res.body["color"], Value::Null);

    let res = alice
        .put(
            "/budget/reorder",
            json!({ "monthId": month_id, "budgetIds": [id(&c), id(&a), id(&b)] }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(ids(&res.body), vec![id(&c), id(&a), id(&b)]);
    assert_eq!(indexes(&res.body), vec![0, 1, 2]);

    let res = alice
        .put("/budget/reorder", json!({ "monthId": month_id }))
        .await;
    assert_eq!(
        res.body,
        json!({ "message": "monthId and budgetIds array are required." })
    );
    let res = alice
        .put(
            "/budget/reorder",
            json!({ "monthId": month_id, "budgetIds": [id(&a), 99999] }),
        )
        .await;
    assert_eq!(
        res.body,
        json!({ "message": "Some budgets do not exist or do not belong to this month." })
    );

    let res = alice.get("/budget").await;
    assert_eq!(ids(&res.body), vec![id(&c), id(&a), id(&b)]);

    // Deleting closes the gap.
    let res = alice.delete(&format!("/budget/{}", id(&a))).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(id(&res.body), id(&a));
    let res = alice.get("/budget").await;
    assert_eq!(ids(&res.body), vec![id(&c), id(&b)]);
    assert_eq!(indexes(&res.body), vec![0, 1]);

    let res = alice.delete(&format!("/budget/{}", id(&a))).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body, json!({ "message": "Budget not found." }));
}

#[sqlx::test]
async fn moving_a_budget_reindexes_both_months(pool: PgPool) {
    let seed = seed(&pool).await;
    let alice = Client::new(&app(pool), "alice");
    let jan = create_month(&alice, seed.year_2025, 1).await;
    let feb = create_month(&alice, seed.year_2025, 2).await;
    let a = create_budget(&alice, jan, "Jan A").await;
    let b = create_budget(&alice, jan, "Jan B").await;
    create_budget(&alice, feb, "Feb A").await;

    let res = alice
        .put(
            &format!("/budget/{}", id(&a)),
            json!({ "description": "Moved", "value": 10, "monthId": feb }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["monthId"], feb);
    assert_eq!(res.body["index"], 1);

    let jan_budgets = alice.get(&format!("/month/{jan}?iBudgets=true")).await;
    assert_eq!(ids(&jan_budgets.body["budgets"]), vec![id(&b)]);
    assert_eq!(indexes(&jan_budgets.body["budgets"]), vec![0]);
}

#[sqlx::test]
async fn income_lifecycle_and_ordering(pool: PgPool) {
    let seed = seed(&pool).await;
    let alice = Client::new(&app(pool), "alice");
    let jan = create_month(&alice, seed.year_2025, 1).await;
    let feb = create_month(&alice, seed.year_2025, 2).await;

    let res = alice
        .post(
            "/income",
            json!({ "value": -5, "createdAt": "2025-01-01T00:00:00Z", "monthId": jan }),
        )
        .await;
    assert_eq!(
        res.body,
        json!({ "message": "Value must be a number and greater than 0." })
    );

    let res = alice
        .post(
            "/income",
            json!({ "value": 10, "createdAt": "2025-01-01T00:00:00Z", "monthId": jan }),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    assert_eq!(res.body["description"], Value::Null);
    let a = res.body;
    let b = create_income(&alice, jan, "Bonus").await;
    let c = create_income(&alice, jan, "Other").await;
    assert_eq!(indexes(&json!([a, b, c])), vec![0, 1, 2]);

    // Omitted description keeps the value; null clears it.
    let res = alice
        .put(
            &format!("/income/{}", id(&b)),
            json!({ "value": 20, "monthId": jan }),
        )
        .await;
    assert_eq!(res.body["description"], "Bonus");
    let res = alice
        .put(
            &format!("/income/{}", id(&b)),
            json!({ "value": 20, "description": null, "monthId": jan }),
        )
        .await;
    assert_eq!(res.body["description"], Value::Null);

    let res = alice
        .put(
            "/income/reorder",
            json!({ "monthId": jan, "incomeIds": [id(&c), id(&b), id(&a)] }),
        )
        .await;
    assert_eq!(ids(&res.body), vec![id(&c), id(&b), id(&a)]);

    let res = alice
        .put(
            "/income/reorder",
            json!({ "monthId": jan, "incomeIds": [id(&c), id(&c)] }),
        )
        .await;
    assert_eq!(
        res.body,
        json!({ "message": "Some incomes do not exist or do not belong to this month." })
    );

    let res = alice
        .put(
            &format!("/income/{}", id(&c)),
            json!({ "value": 5, "monthId": feb }),
        )
        .await;
    assert_eq!(res.body["index"], 0);
    let jan_incomes = alice.get(&format!("/month/{jan}?iIncomes=true")).await;
    assert_eq!(ids(&jan_incomes.body["incomes"]), vec![id(&b), id(&a)]);
    assert_eq!(indexes(&jan_incomes.body["incomes"]), vec![0, 1]);

    let res = alice.delete(&format!("/income/{}", id(&b))).await;
    assert_eq!(id(&res.body), id(&b));
    let jan_incomes = alice.get(&format!("/month/{jan}?iIncomes=true")).await;
    assert_eq!(indexes(&jan_incomes.body["incomes"]), vec![0]);
    assert_eq!(
        alice.delete(&format!("/income/{}", id(&b))).await.status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test]
async fn expense_lifecycle_and_category_ownership(pool: PgPool) {
    let seed = seed(&pool).await;
    let app = app(pool);
    let alice = Client::new(&app, "alice");
    let bob = Client::new(&app, "bob");

    let month_id = create_month(&alice, seed.year_2025, 1).await;
    let budget = id(&create_budget(&alice, month_id, "Groceries").await);
    let bobs_category = id(&bob
        .post("/category", json!({ "name": "Bob only" }))
        .await
        .body);
    let alices_category = id(&alice
        .post("/category", json!({ "name": "Alice only" }))
        .await
        .body);

    let res = alice
        .post("/expense", json!({ "description": "ab", "value": 5, "createdAt": "2025-01-01T00:00:00Z", "budgetId": budget, "categoryId": alices_category }))
        .await;
    assert_eq!(
        res.body,
        json!({ "message": "Title must have at least 3 characters." })
    );

    let res = alice
        .post("/expense", json!({ "description": "Shop", "value": 5, "createdAt": "2025-01-01T00:00:00Z", "budgetId": budget, "categoryId": bobs_category }))
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body, json!({ "message": "Category does not exist." }));

    let res = alice
        .post("/expense", json!({ "description": "Shop", "value": 5, "createdAt": "2025-01-01T00:00:00Z", "budgetId": 99999, "categoryId": alices_category }))
        .await;
    assert_eq!(res.body, json!({ "message": "Budget does not exist." }));

    let res = alice
        .post("/expense", json!({ "description": "Shop", "value": 5, "budgetId": budget, "categoryId": alices_category }))
        .await;
    assert_eq!(
        res.body,
        json!({ "message": "createdAt must be a valid ISO-8601 date-time." })
    );

    let global = create_expense(&alice, budget, seed.global_category as i64).await;
    let own = create_expense(&alice, budget, alices_category).await;

    let res = alice
        .put(&format!("/expense/{}", id(&own)), json!({ "description": "Updated", "value": 9.75, "budgetId": budget, "categoryId": bobs_category }))
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body, json!({ "message": "Category does not exist." }));

    let res = alice
        .put(&format!("/expense/{}", id(&own)), json!({ "description": "Updated", "value": 9.75, "budgetId": budget, "categoryId": seed.global_category }))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["description"], "Updated");
    assert_eq!(res.body["value"], 9.75);
    assert_eq!(res.body["createdAt"], own["createdAt"]);

    let res = alice.get("/expense").await;
    assert_eq!(ids(&res.body), vec![id(&global), id(&own)]);

    let res = alice.delete(&format!("/expense/{}", id(&global))).await;
    assert_eq!(id(&res.body), id(&global));
    let res = alice.delete(&format!("/expense/{}", id(&global))).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body, json!({ "message": "Expense not found." }));
}

#[sqlx::test]
async fn category_crud(pool: PgPool) {
    let seed = seed(&pool).await;
    let app = app(pool);
    let alice = Client::new(&app, "alice");
    let bob = Client::new(&app, "bob");

    let res = alice.post("/category", json!({ "name": "   " })).await;
    assert_eq!(res.body, json!({ "message": "Name is required." }));

    let res = alice.post("/category", json!({ "name": "  Pets  " })).await;
    assert_eq!(res.status, StatusCode::CREATED);
    assert_eq!(res.body["name"], "Pets");
    assert_eq!(res.body["expenseCount"], 0);
    let pets = id(&res.body);

    let res = alice.post("/category", json!({ "name": "Pets" })).await;
    assert_eq!(
        res.body,
        json!({ "message": "Category name must be unique." })
    );
    // Another user may reuse the name.
    assert_eq!(
        bob.post("/category", json!({ "name": "Pets" }))
            .await
            .status,
        StatusCode::CREATED
    );

    let res = alice.get("/category").await;
    let names: Vec<&str> = res
        .body
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Housing", "Pets"]);
    assert_eq!(res.body[0]["userId"], Value::Null);

    let month_id = create_month(&alice, seed.year_2025, 1).await;
    let budget = id(&create_budget(&alice, month_id, "Groceries").await);
    create_expense(&alice, budget, pets).await;
    create_expense(&alice, budget, seed.global_category as i64).await;

    // Bob's view of the global category must not count Alice's expenses.
    let bob_view = bob.get("/category").await;
    assert_eq!(bob_view.body[0]["expenseCount"], 0);
    let alice_view = alice.get("/category").await;
    assert_eq!(alice_view.body[0]["expenseCount"], 1);
    assert_eq!(alice_view.body[1]["expenseCount"], 1);

    let res = alice
        .put(&format!("/category/{pets}"), json!({ "name": "Animals" }))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["name"], "Animals");
    assert_eq!(res.body["expenseCount"], 1);

    let res = alice
        .put(
            &format!("/category/{}", seed.global_category),
            json!({ "name": "Home" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(
        res.body,
        json!({ "message": "Category not found or you do not have permission to edit it." })
    );

    let res = bob.delete(&format!("/category/{pets}")).await;
    assert_eq!(
        res.body,
        json!({ "message": "Category not found or you do not have permission to delete it." })
    );

    let res = alice.delete(&format!("/category/{pets}")).await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        res.body,
        json!({ "message": "Cannot delete category: it is used by 1 expense." })
    );

    let empty = id(&alice
        .post("/category", json!({ "name": "Unused" }))
        .await
        .body);
    let res = alice.delete(&format!("/category/{empty}")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(
        res.body,
        json!({ "success": true, "message": "Category deleted successfully" })
    );
}

#[sqlx::test]
async fn users_cannot_touch_each_others_data(pool: PgPool) {
    let seed = seed(&pool).await;
    let app = app(pool);
    let alice = Client::new(&app, "alice");
    let bob = Client::new(&app, "bob");

    let month_id = create_month(&alice, seed.year_2025, 1).await;
    let budget = id(&create_budget(&alice, month_id, "Groceries").await);
    let income = id(&create_income(&alice, month_id, "Salary").await);
    let expense = id(&create_expense(&alice, budget, seed.global_category as i64).await);

    for path in ["/budget", "/income", "/expense"] {
        assert!(
            bob.get(path).await.body.as_array().unwrap().is_empty(),
            "{path}"
        );
    }
    assert!(
        bob.get(&format!("/month?yearId={}", seed.year_2025))
            .await
            .body
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        bob.get(&format!("/month/{month_id}")).await.status,
        StatusCode::NOT_FOUND
    );

    let month_body = json!({ "value": 2, "description": "hijack" });
    assert_eq!(
        bob.put(&format!("/month/{month_id}"), month_body)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        bob.delete(&format!("/month/{month_id}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        bob.post(
            &format!("/month/{month_id}/copy"),
            json!({ "value": 2, "yearId": seed.year_2025 })
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );

    // Bob cannot attach data to Alice's month or budget.
    let res = bob
        .post("/budget", json!({ "description": "Mine", "value": 1, "createdAt": "2025-01-01T00:00:00Z", "monthId": month_id }))
        .await;
    assert_eq!(res.body, json!({ "message": "Month does not exist." }));
    let res = bob
        .post("/expense", json!({ "description": "Mine", "value": 1, "createdAt": "2025-01-01T00:00:00Z", "budgetId": budget, "categoryId": seed.global_category }))
        .await;
    assert_eq!(res.body, json!({ "message": "Budget does not exist." }));
    let res = bob
        .put(
            "/budget/reorder",
            json!({ "monthId": month_id, "budgetIds": [budget] }),
        )
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);

    // Bob's own month cannot be used to move Alice's records.
    let bobs_month = create_month(&bob, seed.year_2025, 1).await;
    let res = bob
        .put(
            &format!("/budget/{budget}"),
            json!({ "description": "Stolen", "value": 1, "monthId": bobs_month }),
        )
        .await;
    assert_eq!(res.body, json!({ "message": "Budget not found." }));
    let res = bob
        .put(
            &format!("/income/{income}"),
            json!({ "value": 1, "monthId": bobs_month }),
        )
        .await;
    assert_eq!(res.body, json!({ "message": "Income not found." }));
    let bobs_budget = id(&create_budget(&bob, bobs_month, "Bob budget").await);
    let res = bob
        .put(&format!("/expense/{expense}"), json!({ "description": "Stolen", "value": 1, "budgetId": bobs_budget, "categoryId": seed.global_category }))
        .await;
    assert_eq!(res.body, json!({ "message": "Expense not found." }));

    for path in [
        format!("/budget/{budget}"),
        format!("/income/{income}"),
        format!("/expense/{expense}"),
    ] {
        assert_eq!(
            bob.delete(&path).await.status,
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }

    let res = alice
        .get(&format!(
            "/month/{month_id}?iIncomes=true&iBudgets=true&iExpenses=true"
        ))
        .await;
    assert_eq!(res.body["description"], "Month 1");
    assert_eq!(
        res.body["budgets"][0]["expenses"].as_array().unwrap().len(),
        1
    );
    assert_eq!(res.body["incomes"].as_array().unwrap().len(), 1);
}
