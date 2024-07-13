use axum::ServiceExt;
use axum::{
    middleware::{self},
    Router,
};
use dotenv::dotenv;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::{env, error::Error, net::SocketAddr};
use tower_http::{normalize_path::NormalizePathLayer, services::ServeDir, trace::TraceLayer};
use tower_layer::Layer;

mod auth;
mod layout;
mod routers;

#[derive(Clone)]
pub struct AppState {
    pool: Pool<Sqlite>,
}

#[derive(Debug, Clone)]
struct AppContext {
    user_id: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState { pool };

    let app = Router::<AppState>::new()
        .nest("/home", routers::home())
        .nest("/", routers::doggo())
        .fallback_service(ServeDir::new("assets"))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::auth))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    // so that `/foo` and `/foo/` render the same page
    let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

pub struct _FormField<T> {
    _value: T,
    _error: String,
}
impl _FormField<String> {
    fn _empty() -> Self {
        Self {
            _value: "".to_string(),
            _error: "".to_string(),
        }
    }
}
