use crate::AppState;
use askama_axum::Template;
use axum::extract::Query;
use axum::{response::Html, routing::get, Router};
use serde::Deserialize;

#[derive(Template)]
#[template(source = "Hello world", ext = "txt")]
pub struct HelloTemplate {}

#[derive(Template, Deserialize)]
#[template(path = "hello/page.html")]
pub struct HelloNameTemplate {
    name: String,
}

pub fn hello_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/", get(|| async { Html(HelloTemplate {}.to_string()) }))
        .route(
            "/jimmy",
            get(|| async {
                Html(
                    HelloNameTemplate {
                        name: "Jimmy".to_string(),
                    }
                    .to_string(),
                )
            }),
        )
        .route(
            "/name",
            get(|Query(data): Query<HelloNameTemplate>| async {
                Html(HelloNameTemplate { ..data }.to_string())
            }),
        )
}
