use crate::{AppContext, AppState};
use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_client_ip::SecureClientIp;
use chrono::{Duration, Utc};
use uuid::Uuid;

const AUTH_TOKEN_COOKIE_NAME: &str = "best_doggo_auth_token";

pub async fn auth<B>(
    State(state): State<AppState>,
    client_ip: SecureClientIp,
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
    };

    let app_context = AppContext {
        user_id,
        client_ip: Some(client_ip.0),
    };
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
