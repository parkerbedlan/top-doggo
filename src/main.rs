use axum::Router;
use dotenv::dotenv;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::{env, net::SocketAddr};
use tower_http::{services::ServeDir, trace::TraceLayer};

mod routers;

#[derive(Clone)]
pub struct AppState {
    pool: Pool<Sqlite>,
    foo: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::<AppState>::new()
        .nest("/hello", routers::hello())
        .nest("/count", routers::count())
        .nest("/foo", routers::foo())
        .nest("/contacts", routers::contacts())
        .nest("/todo", routers::todo())
        .nest_service("/", ServeDir::new("assets"))
        .with_state(AppState {
            pool,
            foo: "bar".to_string(),
        });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await
        .unwrap();

    Ok(())
}
