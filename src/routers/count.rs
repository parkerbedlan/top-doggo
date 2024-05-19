use askama_axum::Template;
use axum::{response::Html, routing::get, Router};
use std::sync::{Arc, Mutex};

#[derive(Template)]
#[template(path = "count/page.html")]
struct CountTemplate {
    count: i32,
}

pub fn count_router() -> Router {
    let count = Arc::new(Mutex::new(0));
    let count_1 = count.clone();

    Router::new().route(
        "/",
        get(|| async {
            async fn f(count: Arc<Mutex<i32>>) -> Html<String> {
                let mut count = count.lock().unwrap();
                *count += 1;
                Html(CountTemplate { count: *count }.to_string())
            }
            f(count_1).await
        }),
    )
}
