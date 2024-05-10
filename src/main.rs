use askama_axum::Template;
use axum::{extract::Query, response::Html, routing::get, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    count: i32,
}

#[tokio::main]
async fn main() {
    let count = Arc::new(Mutex::new(0));

    let app = Router::new()
        .route(
            "/",
            get(|| async move {
                // let count = count.clone();
                let mut count_guard = count.lock().unwrap();
                *count_guard += 1;
                let count = *count_guard;
                Html(HomeTemplate { count }.render().unwrap())
            }),
        )
        .nest_service("/assets", ServeDir::new("assets"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await
        .unwrap();
}
