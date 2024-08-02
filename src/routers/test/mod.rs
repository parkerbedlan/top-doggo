use crate::{layout::base, AppContext, AppState};
use axum::{extract::State, routing::get, Extension, Router};
use maud::html;

pub fn test_router() -> Router<AppState> {
    Router::<AppState>::new().route(
        "/",
        get(
            |State(_state): State<AppState>, Extension(_context): Extension<AppContext>| async move {
                let response = reqwest::get("https://jsonplaceholder.typicode.com/posts/1").await.expect("The external request should return some json.");
                let response = response.text().await.expect("The content of the response should be able to be parsed as text.");
                base(
                    html! {
                        div class="flex-1 flex flex-col justify-center items-center gap-4" {
                            p {
                                "Hello, world!"
                            }
                            pre {(response)}
                        }
                    },
                    Some("Test".to_string()),
                    None
                )
            },
        ),
    )
}
