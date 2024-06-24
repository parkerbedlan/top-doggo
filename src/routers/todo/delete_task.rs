use crate::AppState;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

pub async fn delete_task(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    sqlx::query!("DELETE FROM task WHERE id=$1", id)
        .execute(&state.pool)
        .await
        .unwrap();
    ()
}
