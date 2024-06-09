use crate::AppState;
use askama_axum::Template;
use axum::{
    extract::{Form, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

// struct FormField<T> {
//     value: T,
//     error: String,
// }

#[derive(Template)]
#[template(path = "todo/page.html")]
struct TodoHomeTemplate {
    baz: String,
}

pub fn todo_router() -> Router<AppState> {
    Router::<AppState>::new().route(
        "/",
        get(|State(state): State<AppState>| async {
            async fn f(state: AppState) -> impl IntoResponse {
                Html(TodoHomeTemplate { baz: state.foo }.to_string())
            }
            f(state).await
        }),
    )
}
