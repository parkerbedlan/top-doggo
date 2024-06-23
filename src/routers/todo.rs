use crate::{base, AppState};
use axum::{
    extract::State,
    response::Html,
    routing::{get, patch},
    Form, Router,
};
use maud::{html, Markup, Render};
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
impl Render for Task {
    fn render(&self) -> Markup {
        html! {
            div class="flex gap-2 w-full items-center"
            {
                input
                    id={"checkbox-task-" (self.id)}
                    type="checkbox"
                    checked[self.done]
                    hx-trigger="change"
                    hx-patch="/todo"
                    hx-vals="js:{checked: event.target.checked, id: Number(event.target.id.split('-')[2])}"
                    hx-swap="none"
                    ;
                label
                    for={"checkbox-task-" (self.id)}
                    {(self.description)}
                div {"hi"}
            }
        }
    }
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
                    base(
                        html! {
                            @for task in tasks.iter() {
                                (task)
                            }
                        },
                        None,
                        None,
                    )
                    .into_string(),
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
