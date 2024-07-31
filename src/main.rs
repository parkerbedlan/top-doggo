use axum::extract::DefaultBodyLimit;
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
    client_ip: Option<std::net::IpAddr>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    // FOR PROD make sure this is not commented out
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState { pool };

    let app = Router::new()
        .nest("/leaderboard", routers::leaderboard())
        .nest("/", routers::doggo())
        .nest("/upload", routers::upload())
        .nest("/", routers::me())
        .fallback_service(ServeDir::new("assets"))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::auth))
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10)) // 10 MiB
        // only necessary if running the app without a proxy like traefik
        // .layer(SecureClientIpSource::ConnectInfo.into_extension())
        .with_state(state);

    // so that `/foo` and `/foo/` render the same page
    let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();

    Ok(())
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
