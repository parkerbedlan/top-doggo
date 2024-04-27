use askama_axum::Template;
use axum::{extract::Query, response::Html, routing::get, Router};
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Template, Deserialize)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

#[derive(Deserialize)]
struct PlusOneParameters {
    before: i32,
}

#[derive(Template)]
#[template(path = "plus1.html")]
struct PlusOneTemplate {
    result: i32,
}

#[tokio::main]
async fn main() {
    // Route all requests on "/" endpoint to anonymous handler.
    //
    // A handler is an async function which returns something that implements
    // `axum::response::IntoResponse`.

    let name = "Jimmy";

    let app = Router::new()
        .route("/simple", get(|| async { "Hello, there!" }))
        .route(
            "/hello",
            get(HelloTemplate {
                name: String::from(name),
            }
            .render()
            .unwrap()),
        )
        .route(
            "/hello2",
            get(|x: Query<HelloTemplate>| async move {
                HelloTemplate {
                    name: String::from(&x.name),
                }
                .render()
                .unwrap()
            }),
        )
        .route(
            "/plus1",
            get(|params: Query<PlusOneParameters>| async move {
                let result = params.before + 1;
                PlusOneTemplate { result }.render().unwrap()
            }),
        )
        .route("/rando", get(rando_handler))
        .route("/from-file", get(from_file_handler));

    // Address that server will bind to.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("listening on {}", addr);
    // Use `hyper::server::Server` which is re-exported through `axum::Server` to serve the app.
    axum::Server::bind(&addr)
        // Hyper server takes a make service.
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// `Deserialize` need be implemented to use with `Query` extractor.
#[derive(Deserialize)]
struct RangeParameters {
    start: usize,
    end: usize,
}

async fn rando_handler(Query(range): Query<RangeParameters>) -> Html<String> {
    // (query: Query<RangeParameters>)
    // let Query { range } = query;

    // Generate a random number in range parsed from query.
    let random_number = thread_rng().gen_range(range.start..range.end);

    // Send response in html format.
    Html(format!(
        "<h1 style=\"color:{};\">Random Number: {}</h1>",
        "blue", random_number
    ))
}

async fn from_file_handler() -> Html<&'static str> {
    Html(include_str!("./example-file.html"))
}
