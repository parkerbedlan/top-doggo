use crate::{base, AppContext, AppState};
use axum::{extract::State, response::Html, routing::get, Extension, Router};
use maud::{html, Markup, Render};

pub fn home_router() -> Router<AppState> {
    Router::<AppState>::new().route(
        "/",
        get(
            |State(_state): State<AppState>, Extension(context): Extension<AppContext>| async move {
                println!("hi");
                Html(
                    base(
                        html! {
                            h1 {"user_id: " (context.user_id)}
                            hr class="mb-4 border-black border-b" ;
                            h1 {"Home"}
                        },
                        None,
                        None,
                    )
                    .into_string(),
                )
            },
        ),
    )
}
