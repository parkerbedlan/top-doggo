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
use sqlx::prelude::FromRow;
use std::sync::{Arc, Mutex};

// struct FormField<T> {
//     value: T,
//     error: String,
// }

#[derive(FromRow)]
struct Task {
    id: u32,
    description: String,
    done: bool,
}

#[derive(Template)]
#[template(path = "todo/page.html")]
struct TodoHomeTemplate {
    baz: String,
    tasks: Vec<Task>,
}

pub fn todo_router() -> Router<AppState> {
    Router::<AppState>::new().route(
        "/",
        get(|State(state): State<AppState>| async {
            async fn f(state: AppState) -> impl IntoResponse {
                let q = "SELECT * FROM task";
                let query = sqlx::query_as::<_, Task>(q);
                let tasks = query.fetch_all(&state.pool).await.unwrap_or(vec![]);
                Html(
                    TodoHomeTemplate {
                        baz: state.foo,
                        tasks,
                    }
                    .to_string(),
                )
            }
            f(state).await
        }),
    )
}
