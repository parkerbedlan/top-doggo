// use askama_axum::Template;
use crate::AppState;
use askama::Template;
use axum::{response::Html, routing::get, Router};

#[derive(Template)]
#[template(path = "foo/page.html")]
struct FooTemplate {}

#[derive(Template)]
#[template(path = "foo/cool-title.html")]
struct CoolTitleTemplate {
    text: String,
}

// template for askama componentization
pub fn foo_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/",
            get(|| async {
                async fn f() -> Html<String> {
                    Html(FooTemplate {}.to_string())
                }
                f().await
            }),
        )
        .route(
            "/title-only",
            get(|| async {
                async fn f() -> Html<String> {
                    Html(
                        CoolTitleTemplate {
                            text: "hiya".to_string(),
                        }
                        .to_string(),
                    )
                }
                f().await
            }),
        )
}
