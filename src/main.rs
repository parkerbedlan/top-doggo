use axum::{
    extract::Extension,
    http::{self, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use dotenv::dotenv;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::{env, error::Error, net::SocketAddr};
use tower_http::{services::ServeDir, trace::TraceLayer};

mod routers;

#[derive(Clone)]
pub struct AppState {
    pool: Pool<Sqlite>,
    // foo: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::<AppState>::new()
        .nest("/todo", routers::todo())
        .nest_service("/", ServeDir::new("assets"))
        .with_state(AppState {
            pool,
            // foo: "bar".to_string(),
        })
        .route_layer(middleware::from_fn(auth))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[derive(Debug, Clone)]
struct Foo {
    bar: String,
}

async fn auth<B: 'static>(mut req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let foo = Foo {
        bar: "baz".to_string(),
    };
    req.extensions_mut().insert(foo);
    Ok(next.run(req).await)
}

pub fn base(content: Markup, title: Option<String>, head: Option<Markup>) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en";
        head {
            // https://www.srihash.org/
            // https://htmx.org/docs/#installing
            script src="https://unpkg.com/htmx.org@2.0.0" integrity="sha384-wS5l5IKJBvK6sPTKa2WZ1js3d947pvWXbPJ1OmWfEuxLgeHcEbjUUA5i9V5ZkpCw" crossorigin="anonymous" {}
            script {(PreEscaped(r#"
                        document.addEventListener("DOMContentLoaded", () => {
                            htmx.config.useTemplateFragments = true;
                            // https://htmx.org/events/
                            // htmx.logAll() in console to see all the events as they happen!
                            document.body.addEventListener("htmx:beforeSwap", (event) => {
                                if (event.detail.xhr.status === 422) {
                                    event.detail.shouldSwap = true;
                                    // suppresses error logging in the console
                                    event.detail.isError = false;
                                }
                            })
                        })
                "#))}
            // script src="https://cdn.tailwindcss.com" {}
            link rel="stylesheet" href="/output.css";
            title {(title.unwrap_or("Welcome".to_string())) " - Acme"}
            (head.unwrap_or(html!{}))
        }
        body {
            div id="content" class="max-w-screen-2xl mx-auto px-4 min-h-screen" hx-boost="true" {(content)}
        }
    }
}

pub struct FormField<T> {
    value: T,
    error: String,
}
impl FormField<String> {
    fn empty() -> Self {
        Self {
            value: "".to_string(),
            error: "".to_string(),
        }
    }
}
