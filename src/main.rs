use askama_axum::Template;
use axum::routing::post;
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

#[derive(Template)]
#[template(path = "count.html")]
struct CountTemplate {
    count: i32,
}

#[tokio::main]
async fn main() {
    let count = Arc::new(Mutex::new(0));
    let count_1 = count.clone();
    let count_2 = count.clone();

    let app = Router::new()
        .route(
            "/",
            get(|| async {
                async fn f(count: Arc<Mutex<i32>>) -> Html<String> {
                    let count = count.lock().unwrap();
                    Html(HomeTemplate { count: *count }.render().unwrap())
                }
                f(count_1).await
            }),
        )
        .route(
            "/count",
            post(|| async {
                async fn f(count: Arc<Mutex<i32>>) -> Html<String> {
                    let mut count = count.lock().unwrap();
                    *count += 1;
                    // Html(count.to_string())
                    Html(CountTemplate { count: *count }.render().unwrap())
                }
                f(count_2).await
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
