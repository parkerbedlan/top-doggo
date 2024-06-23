use crate::{base, AppState};
use axum::{
    extract::{Path, State},
    response::Html,
    routing::{delete, get, patch, post},
    Form, Router,
};
use maud::{html, Markup, Render};
use serde::Deserialize;

struct FormField<T> {
    value: T,
    error: String,
}
impl FormField<String> {
    fn empty() -> Self {
        Self {
            value: "".to_string(),
            error: "".to_string(),
        }
    }
}

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
                button
                    class="text-error"
                    hx-delete={"/todo/" (self.id)}
                    hx-target="closest div"
                    hx-swap="outerHTML"
                    {"X"}
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
            }
        }
    }
}

#[derive(Deserialize)]
struct PatchTodoParams {
    id: i64,
    checked: bool,
}

#[derive(Deserialize)]
struct PostTodoParams {
    description: String,
}

fn create_form(description: FormField<String>) -> Markup {
    html! {
        form
            hx-post="/todo"
            hx-swap="outerHTML"
            class="flex"
            {
                div
                    class="flex flex-col"
                    {
                        input
                            type="text"
                            id="description"
                            name="description"
                            placeholder="New task"
                            value=(description.value)
                            .input .input-bordered .border-error[description.error != ""]
                            ;
                        label
                            for="description"
                            class="label-text-alt text-error"
                            {(description.error)}
                    }
                button
                    class="btn"
                    type="submit"
                    {"Submit"}
            }
    }
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
                            div class="flex flex-col gap-8" {
                                div
                                    id="tasks"
                                    class="flex flex-col gap-2"
                                    {
                                        @for task in tasks.iter() {
                                            (task)
                                        }
                                    }
                                (create_form(FormField::empty()))
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
        .route(
            "/",
            post(
                |State(state): State<AppState>, Form(form): Form<PostTodoParams>| async move {
                    if form.description == "" {
                        return Html(
                            create_form(FormField {
                                value: form.description,
                                error: "Required".to_string(),
                            })
                            .into_string(),
                        );
                    }

                    let new_task = sqlx::query_as!(
                        Task,
                        "INSERT INTO task (description) VALUES ($1) RETURNING *",
                        form.description
                    )
                    .fetch_one(&state.pool)
                    .await
                    .unwrap();

                    Html(
                        html! {
                            (create_form(FormField::empty()))
                            div
                                hx-swap-oob="beforeend:#tasks"
                                {
                                    (new_task)
                                }
                        }
                        .into_string(),
                    )
                },
            ),
        )
        .route(
            "/:id",
            delete(
                |State(state): State<AppState>, Path(id): Path<i64>| async move {
                    sqlx::query!("DELETE FROM task WHERE id=$1", id)
                        .execute(&state.pool)
                        .await
                        .unwrap();
                    ()
                },
            ),
        )
}
