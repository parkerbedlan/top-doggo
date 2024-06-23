use super::{create, Task};
use crate::{base, AppState, FormField};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use maud::html;

pub async fn todo_home(State(state): State<AppState>) -> impl IntoResponse {
    let tasks = sqlx::query_as!(Task, "SELECT * FROM task")
        .fetch_all(&state.pool)
        .await
        .unwrap_or(vec![]);
    Html(
        base(
            html! {
                div class="flex flex-col gap-8" {
                    div
                        id="tasks"
                        class="flex flex-col gap-2"
                        {
                            @for task in tasks.iter() {
                                (task)
                            }
                        }
                    (create::new_task_form(FormField::empty()))
                }
            },
            None,
            None,
        )
        .into_string(),
    )
}
