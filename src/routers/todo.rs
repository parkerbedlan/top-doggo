use crate::AppState;
use askama_axum::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use sqlx::prelude::FromRow;

// struct FormField<T> {
//     value: T,
//     error: String,
// }

#[derive(FromRow)]
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

pub fn todo_router() -> Router<AppState> {
    Router::<AppState>::new().route(
        "/",
        get(|State(state): State<AppState>| async {
            async fn f(state: AppState) -> impl IntoResponse {
                // let tasks = sqlx::query_as::<_, Task>("SELECT * FROM task")
                //     .fetch_all(&state.pool)
                //     .await
                //     .unwrap_or(vec![]);
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
            }
            f(state).await
        }),
    )
}
