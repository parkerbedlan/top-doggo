use super::doggo::xp::xp_section;
use crate::{
    auth::{create_new_auth_cookie, create_new_auth_token}, layout::{base, NavLink}, routers::doggo::xp::get_xp, AppContext, AppState, FormField
};
use axum::{
    extract::{Query, State}, http::{header, HeaderMap, StatusCode}, response::Html, routing::{get, post}, Extension, Form, Router
};
use lettre::{
    address::AddressError,
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use std::env;
use uuid::Uuid;

pub fn me_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/me",
            get(|State(state): State<AppState>, Extension(context): Extension<AppContext>, Query(params): Query<MeParams>| async move {
                base(
                    me_page_content(state, context, params).await,
                    Some("Me".to_string()),
                    Some(NavLink::Me),
                )
            }),
        )
        .route("/me-refresh", get(|State(state): State<AppState>, Extension(context): Extension<AppContext>, Query(params): Query<MeParams>| async move {
            Html(me_page_content(state, context, params).await.into_string())
        }))
        .route(
            "/send-magic-link",
            post(|State(state): State<AppState>, Extension(context): Extension<AppContext>, Form(form): Form<SendMagicLinkFormParams>| async move {
                println!("Sending email...");

                let err = |form_error: &str| {
                    Html(
                        send_magic_link_form(
                            FormField {
                                value: form.email_address.clone(),
                                error: form_error.to_string(),
                            },
                        )
                        .into_string(),
                    )
                };

                // TODO: rate limit the email

                let email_sent = send_magic_link_email(&state.pool, &form.email_address).await;
                if email_sent.is_err() {
                    return err("Invalid Email");
                }

                let client_ip: Option<String> = context.client_ip.map(|ip| ip.to_string());
                let _ = sqlx::query!("INSERT INTO log (action, user_id, client_ip, notes) VALUES ('send-magic-link', $1, $2, $3)", context.user_id, client_ip, form.email_address)
                    .fetch_one(&state.pool).await;

                return Html(email_sent_message().into_string())
            })
        )
        .route("/login", get(|
            State(state): State<AppState>,
            Extension(context): Extension<AppContext>,
            Query(params): Query<LoginParams>
        | async move {
            println!("Logging in...");
            println!("token: {}", params.token);

            let token_email = sqlx::query!("SELECT email FROM email_token WHERE token=$1 AND created_at > datetime('now', '-30 minutes')", params.token)
                .fetch_optional(&state.pool)
                .await.unwrap();
            if token_email.is_none() {
                return (
                    StatusCode::TEMPORARY_REDIRECT,
                    {
                        let mut headers = HeaderMap::new();
                        headers.insert(header::LOCATION, "/sorry?reason=expired_or_does_not_exist".parse().unwrap());
                        headers
                    }
                )
            }
            let token_email = token_email.unwrap().email;

            if let Some(current_email) = context.user_email {
                if current_email != token_email {
                    return (
                        StatusCode::TEMPORARY_REDIRECT,
                        {
                            let mut headers = HeaderMap::new();
                            headers.insert(header::LOCATION, "/sorry?reason=already_logged_in".parse().unwrap());
                            headers
                        }
                    )
                } else {
                    return (
                        StatusCode::TEMPORARY_REDIRECT,
                        {
                            let mut headers = HeaderMap::new();
                            headers.insert(header::LOCATION, "/me".parse().unwrap());
                            headers
                        }
                    )
                }
            }
            
            let existing_user = sqlx::query!("SELECT id FROM user WHERE email = $1", token_email)
                .fetch_optional(&state.pool).await.unwrap();

            if let Some(existing_user) = existing_user {
                // log in
                let new_auth_token = create_new_auth_token(&state.pool, existing_user.id).await;

                let client_ip: Option<String> = context.client_ip.map(|ip| ip.to_string());
                let notes = format!("{} {}", token_email, context.user_id);
                let _ = sqlx::query!("INSERT INTO log (action, user_id, client_ip, notes) VALUES ('log-in', $1, $2, $3)", existing_user.id, client_ip, notes)
                    .fetch_one(&state.pool).await;

                return (
                    StatusCode::TEMPORARY_REDIRECT,
                    {
                        let mut headers = HeaderMap::new();
                        headers.insert(header::LOCATION, "/me".parse().unwrap());
                        headers.insert(header::SET_COOKIE, create_new_auth_cookie(new_auth_token).parse().unwrap());
                        headers
                    }
                )
            } else {
               // sign up
                let _ = sqlx::query!("UPDATE user SET email = $1, total_xp = total_xp + 2000 WHERE id = $2", token_email, context.user_id).fetch_one(&state.pool).await;

                let client_ip: Option<String> = context.client_ip.map(|ip| ip.to_string());
                let _ = sqlx::query!("INSERT INTO log (action, user_id, client_ip, notes) VALUES ('sign-up', $1, $2, $3)", context.user_id, client_ip, token_email)
                    .fetch_one(&state.pool).await;


                return (
                    StatusCode::TEMPORARY_REDIRECT,
                    {
                        let mut headers = HeaderMap::new();
                        headers.insert(header::LOCATION, "/me?new_user=true".parse().unwrap());
                        headers
                    }
                )


            }
        }))
        .route("/sorry", get(|Extension(context): Extension<AppContext>, Query(params): Query<SorryParams>| async move {
            let message = match params.reason {
                SorryReason::ExpiredOrDoesNotExist => "That token is expired or doesn't exist.".to_string(),
                SorryReason::AlreadyLoggedIn => format!("You're already logged in with {}", context.user_email.unwrap_or("another email".to_string()))
            };

            base(
                html! {
                    div class="flex-1 flex flex-col gap-4 items-center justify-center" {
                        h1 class="text-5xl" {"Oops..."}
                        h3 class="text-3xl" { (message) }
                        a class="text-3xl underline text-primary" href="/" {"Back to the dog show"}
                    }
                },
                None,
                None
                )
        }))
}

async fn send_email(to_mailbox: Mailbox, subject: &str, content: Markup) -> Result<(), ()> {
    let mode = env::var("MODE").unwrap();
    if mode == "development" {
        println!("Email that would be sent: {:?}{:?}", to_mailbox, content);
        return Ok(())
    }

    let email = Message::builder()
        .from(
            "Top Doggo <parkerbedlan@gmail.com>"
                .to_string()
                .parse()
                .unwrap(),
        )
        .to(to_mailbox)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(content.into_string())
        .unwrap();

    let creds = Credentials::new(
        env::var("SMTP_USERNAME").expect("SMTP Username not specified "),
        env::var("SMTP_PASSWORD").expect("SMTP Password not specified"),
    );

    let mailer = SmtpTransport::relay(&env::var("SMTP_HOST").expect("SMTP Host not specified"))
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => {
            println!("Email failed to send: {}", e);
            return Err(());
        }
    }

    Ok(())
}
async fn send_magic_link_email(pool: &Pool<Sqlite>, to_email_address: &str) -> Result<(), ()> {
    let to_mailbox: Result<Mailbox, AddressError> =
        format!("Top Doggo Judge <{}>", to_email_address).parse();
    if to_mailbox.is_err() {
        return Err(());
    }
    let to_mailbox = to_mailbox.unwrap();

    let magic_token = Uuid::new_v4().to_string();
    let _ = sqlx::query!(
        "INSERT INTO email_token (token, email) VALUES ($1, $2)",
        magic_token,
        to_email_address
    )
    .fetch_one(pool)
    .await;

    send_email(to_mailbox, "Top Doggo - Your Magic Link",
        html!{
            h1 {"Magic Link for Top Doggo"}
            h3 {"Follow this link to log in to the platform:"}
            a href={ (env::var("BASE_URL").unwrap()) "/login?token=" (magic_token)} style="font-size: 1.5rem;" {"Log In"}
        }
    ).await
}

#[derive(Deserialize)]
struct SendMagicLinkFormParams {
    email_address: String,
}
fn send_magic_link_form(email_address: FormField<String>) -> Markup {
    html! {
    form
        hx-post="/send-magic-link"
        hx-swap="outerHTML"
        hx-target="this"
        class="gap-4 flex flex-col items-center"
    {
        div class="flex gap-1 flex-wrap text-center justify-center" {
            p {"Log in with email to save your progress! "}
            p {(PreEscaped("&nbsp;"))"( and earn 2000xp the first time :O )"}
        }
        div class="flex gap-2 flex-wrap max-w-screen-sm justify-center" {
            div class="flex flex-col max-w-72 min-w-52 basis-52 shrink grow items-start" {
                input type="email"
                    name="email_address"
                    id="email_address"
                    placeholder="dogfan@example.com"
                    class={ "input input-bordered w-full text-lg" @if !email_address.error.is_empty() {" !border-error"} }
                    value=(email_address.value)
                    ;
                label for="email_address" class="text-lg text-error leading-tight" {(email_address.error)}
            }
            button type="submit" hx-swap="none" class="btn btn-primary" {"Log In using Magic Link"}
        }
    }}
}

#[derive(Deserialize)]
struct LoginParams {
    token: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SorryReason {
    ExpiredOrDoesNotExist,
    AlreadyLoggedIn
}

#[derive(Deserialize)]
struct SorryParams {
    reason: SorryReason
}


#[derive(Deserialize)]
struct MeParams {
    new_user: Option<bool>
}
async fn me_page_content(state: AppState, context: AppContext, params: MeParams) -> Markup {
    let recently_sent_magic_link = sqlx::query!("SELECT COUNT(*) AS recently_sent FROM log WHERE action='send-magic-link' AND user_id=$1 AND created_at > datetime('now', '-5 minutes')", context.user_id)
        .fetch_one(&state.pool).await.unwrap().recently_sent == 1;

    html! {
        div
            hx-get="/me-refresh"
            hx-target="this"
            hx-swap="outerHTML"
            hx-trigger="me-refresh"
            _="on visibilitychange from document if document.visibilityState is 'visible' send 'me-refresh' end"
            class="flex-1 flex flex-col items-center justify-center gap-20 text-center" 
        {
            @if let Some(email) = context.user_email {
                h1 class="text-2xl"
                {"You're currently logged in with the email "(email)" :)"}
            } @else if recently_sent_magic_link {
                (email_sent_message())
            } @else {
                (send_magic_link_form(FormField::empty()))
            }
            (xp_section(
                get_xp(&state.pool, context.user_id).await, 
                if params.new_user.unwrap_or(false) {Some(2000)} else {None},
                false)
            )
            a href="/leaderboard/top/personal" class="underline text-primary text-lg" {"Your personal leaderboard"}
        }
    }
}

fn email_sent_message() -> Markup {
    html! {
        h1 class="text-5xl" { "Email sent. Check your inbox!" }
    }
}
