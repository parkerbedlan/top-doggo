use axum::{extract::State, response::IntoResponse, Form};
use serde::Deserialize;

use crate::AppState;

#[derive(Deserialize)]
pub struct UpdateTaskParams {
    id: i64,
    checked: bool,
}

pub async fn update_task(
    State(state): State<AppState>,
    Form(form): Form<UpdateTaskParams>,
) -> impl IntoResponse {
    sqlx::query!("UPDATE task SET done=$1 WHERE id=$2", form.checked, form.id)
        .execute(&state.pool)
        .await
        .unwrap();
    ()
}
