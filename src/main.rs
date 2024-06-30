use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    Router,
};
use chrono::{Duration, Utc};
use dotenv::dotenv;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::{env, error::Error, net::SocketAddr};
use tower_http::{services::ServeDir, trace::TraceLayer};
use uuid::Uuid;

mod routers;

#[derive(Clone)]
pub struct AppState {
    pool: Pool<Sqlite>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", Uuid::new_v4().to_string());

    dotenv().ok();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState { pool };

    let app = Router::<AppState>::new()
        .nest("/", routers::home())
        .nest("/todo", routers::todo())
        .fallback_service(ServeDir::new("assets"))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[derive(Debug, Clone)]
struct AppContext {
    user_id: i64,
}

const AUTH_TOKEN_COOKIE_NAME: &str = "best_doggo_auth_token";

async fn auth<B>(
    State(state): State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let original_auth_token = req
        .headers()
        .get(http::header::COOKIE)
        .and_then(|cookie_header| {
            cookie_header.to_str().ok().and_then(|cookie_str| {
                cookie_str.split(';').find_map(|cookie| {
                    let mut parts = cookie.trim().splitn(2, '=');
                    if parts.next() == Some(AUTH_TOKEN_COOKIE_NAME) {
                        parts.next().map(|value| value.to_string())
                    } else {
                        None
                    }
                })
            })
        })
        .unwrap_or_default();

    let mut new_auth_token: Option<String> = None;

    let user_id = match sqlx::query_as!(
        AppContext,
        "SELECT user_id FROM session WHERE token = $1",
        original_auth_token
    )
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(AppContext { user_id })) => user_id,
        _ => {
            let new_user = sqlx::query!("INSERT INTO user DEFAULT VALUES RETURNING id")
                .fetch_one(&state.pool)
                .await
                .unwrap();
            let new_user_id = new_user.id;

            let new_token = Uuid::new_v4().to_string();
            let _ = sqlx::query!(
                "INSERT INTO session (token, user_id) VALUES ($1, $2)",
                new_token,
                new_user_id
            )
            .fetch_one(&state.pool)
            .await;

            new_auth_token = Some(new_token);

            new_user_id
        }
    };

    let app_context = AppContext { user_id };
    req.extensions_mut().insert(app_context);

    let mut response = next.run(req).await;

    if let Some(token) = new_auth_token {
        // Set the updated cookie in the response
        let expiration = Utc::now() + Duration::days(365 * 10);
        let expiration = expiration.format("%a, %d %b %Y %H:%M:%S GMT");
        let new_cookie = format!(
            "{}={}; Path=/; HttpOnly; Secure; SameSite=Strict; Expires={}",
            AUTH_TOKEN_COOKIE_NAME, token, expiration
        );
        response
            .headers_mut()
            .insert(http::header::SET_COOKIE, new_cookie.parse().unwrap());
    }

    Ok(response)
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
