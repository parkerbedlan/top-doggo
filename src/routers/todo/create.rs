use super::Task;
use crate::{AppState, FormField};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    Form,
};
use maud::{html, Markup};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateTaskParams {
    description: String,
}

pub fn new_task_form(description: FormField<String>) -> Markup {
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
                            .input .input-bordered ."!border-error"[description.error != ""]
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

pub async fn create_task(
    State(state): State<AppState>,
    Form(form): Form<CreateTaskParams>,
) -> impl IntoResponse {
    if form.description == "" {
        return Html(
            new_task_form(FormField {
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
            (new_task_form(FormField::empty()))
            div
                hx-swap-oob="beforeend:#tasks"
                {
                    (new_task)
                }
        }
        .into_string(),
    )
}
