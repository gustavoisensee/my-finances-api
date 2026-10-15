pub mod access_token;
pub mod budget;
pub mod category;
pub mod expense;
pub mod fields;
pub mod health;
pub mod income;
pub mod month;
pub mod user;
pub mod year;

use axum::Router;
use axum::routing::{get, post, put};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(health::index))
        .route("/status", get(health::status))
        .route("/year", get(year::list))
        .route("/category", get(category::list).post(category::create))
        .route(
            "/category/{id}",
            put(category::update).delete(category::remove),
        )
        .route("/budget", get(budget::list).post(budget::create))
        .route("/budget/reorder", put(budget::reorder))
        .route("/budget/{id}", put(budget::update).delete(budget::remove))
        .route("/expense", get(expense::list).post(expense::create))
        .route(
            "/expense/{id}",
            put(expense::update).delete(expense::remove),
        )
        .route("/income", get(income::list).post(income::create))
        .route("/income/reorder", put(income::reorder))
        .route("/income/{id}", put(income::update).delete(income::remove))
        .route("/month", get(month::list).post(month::create))
        .route(
            "/month/{id}",
            get(month::get_one).put(month::update).delete(month::remove),
        )
        .route("/month/{id}/copy", post(month::copy))
        .route("/access-token", get(access_token::list))
        .route("/user", get(user::list).post(user::create))
        .route("/user/{id}", put(user::update).delete(user::remove))
}
