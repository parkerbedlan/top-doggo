use crate::{AppContext, AppState};
use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_client_ip::XForwardedFor;
use chrono::{Duration, Utc};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

// probably not worth renaming (it would sign everybody out)
const AUTH_TOKEN_COOKIE_NAME: &str = "best_doggo_auth_token";

pub async fn auth<B>(
    State(state): State<AppState>,
    // secure_client_ip: SecureClientIp,
    XForwardedFor(client_ips): XForwardedFor,
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
    // println!("original auth token: {}", original_auth_token);

    let mut new_auth_token: Option<String> = None;

    let user_id = sqlx::query!(
        "SELECT user_id FROM session WHERE token = $1",
        original_auth_token
    )
    .fetch_optional(&state.pool)
    .await;
    let user_id = if let Ok(Some(record)) = user_id {
        record.user_id
    } else {
        let new_user = sqlx::query!("INSERT INTO user DEFAULT VALUES RETURNING id")
            .fetch_one(&state.pool)
            .await
            .unwrap();
        let new_user_id = new_user.id;

        new_auth_token = Some(create_new_auth_token(&state.pool, new_user_id).await);

        new_user_id
    };

    let user_email: Option<String> = sqlx::query!("SELECT email FROM user WHERE id=$1", user_id)
        .fetch_one(&state.pool)
        .await
        .unwrap()
        .email;

    // let client_ip = Some(secure_client_ip.0);
    let client_ip = client_ips.first().cloned();

    let app_context = AppContext {
        user_id,
        user_email,
        client_ip,
    };
    req.extensions_mut().insert(app_context);

    let mut response = next.run(req).await;

    if let Some(token) = new_auth_token {
        // Set the updated cookie in the response
        response.headers_mut().insert(
            http::header::SET_COOKIE,
            create_new_auth_cookie(token).parse().unwrap(),
        );
    }

    Ok(response)
}

pub fn create_new_auth_cookie(token: String) -> String {
    let expiration = Utc::now() + Duration::days(365 * 10);
    let expiration = expiration.format("%a, %d %b %Y %H:%M:%S GMT");
    format!(
        "{}={}; Path=/; HttpOnly; Secure; SameSite=Strict; Expires={}",
        AUTH_TOKEN_COOKIE_NAME, token, expiration
    )
}

pub async fn create_new_auth_token(pool: &Pool<Sqlite>, user_id: i64) -> String {
    let new_token = Uuid::new_v4().to_string();
    let _ = sqlx::query!(
        "INSERT INTO session (token, user_id) VALUES ($1, $2)",
        new_token,
        user_id
    )
    .fetch_one(pool)
    .await;
    return new_token;
}
