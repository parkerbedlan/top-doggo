use crate::AppState;
use askama_axum::Template;
use axum::{
    extract::State,
    response::Html,
    routing::{get, patch},
    Form, Router,
};
use serde::Deserialize;

// struct FormField<T> {
//     value: T,
//     error: String,
// }

// #[derive(FromRow)]
struct Task {
    id: i64,
    description: String,
    done: bool,
}

#[derive(Template)]
#[template(path = "todo/page.html")]
struct TodoHomeTemplate {
    baz: String,
    tasks: Vec<Task>,
}

#[derive(Deserialize)]
struct PatchTodoParams {
    id: i64,
    checked: bool,
}

pub fn todo_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/",
            get(|State(state): State<AppState>| async move {
                let tasks = sqlx::query_as!(Task, "SELECT * FROM task")
                    .fetch_all(&state.pool)
                    .await
                    .unwrap_or(vec![]);
                Html(
                    TodoHomeTemplate {
                        baz: state.foo,
                        tasks,
                    }
                    .to_string(),
                )
            }),
        )
        .route(
            "/",
            patch(
                |State(state): State<AppState>, Form(form): Form<PatchTodoParams>| async move {
                    sqlx::query!("UPDATE task SET done=$1 WHERE id=$2", form.checked, form.id)
                        .execute(&state.pool)
                        .await
                        .unwrap();
                    ()
                },
            ),
        )
}
