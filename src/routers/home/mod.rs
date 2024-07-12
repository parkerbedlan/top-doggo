use crate::{base, AppContext, AppState};
use axum::{extract::State, routing::get, Extension, Router};
use maud::html;

pub fn home_router() -> Router<AppState> {
    Router::<AppState>::new().route(
        "/",
        get(
            |State(_state): State<AppState>, Extension(context): Extension<AppContext>| async move {
                base(html! {
                    h1 {"user_id: " (context.user_id)}
                    hr class="mb-4 border-black border-b" ;
                    h1 {"Home"}
                })
            },
        ),
    )
}
