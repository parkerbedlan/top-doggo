use super::create_task::new_task_form;
use super::Task;
use crate::{base, AppContext, AppState, FormField};
use axum::{
    debug_handler,
    extract::State,
    response::{Html, IntoResponse},
    Extension,
};
use maud::html;

#[debug_handler]
pub async fn todo_home(
    State(state): State<AppState>,
    Extension(context): Extension<AppContext>,
) -> impl IntoResponse {
    println!("{:?}", context.user_id);
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
                    (new_task_form(FormField::empty()))
                }
            },
            None,
            None,
        )
        .into_string(),
    )
}
