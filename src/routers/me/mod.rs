use super::doggo::xp::xp_section;
use crate::{
    layout::{base, NavLink},
    routers::doggo::xp::get_xp,
    AppContext, AppState, FormField,
};
use axum::{
    extract::{Path, Query, State},
    response::Html,
    routing::{get, post},
    Extension, Form, Router,
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
use std::{env, str::FromStr};
use uuid::Uuid;

pub fn me_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/me",
            get(|State(state): State<AppState>, Extension(context): Extension<AppContext>| async move {
                base(
                    html! {
                        div class="flex-1 flex flex-col items-center justify-center gap-20 text-center" {
                            (send_magic_link_form(FormField::empty()))
                            (xp_section(get_xp(&state.pool, context.user_id).await, None, false))
                            a href="/leaderboard/top/personal" class="underline text-primary" {"Your personal leaderboard"}
                        }
                    },
                    Some("Me".to_string()),
                    Some(NavLink::Me),
                )
            }),
        )
        .route(
            "/send-magic-link",
            post(|State(state): State<AppState>, Form(form): Form<SendMagicLinkFormParams>| async move {
                println!("Sending email...");

                let email_sent = send_magic_link_email(&state.pool, &form.email_address).await;
                if email_sent.is_err() {
                    return Html(
                        send_magic_link_form(FormField{value: form.email_address, error: "Invalid Email".to_string()})
                    .into_string());
                }

                return Html(html! {
                    h1 class="text-5xl" { "Email sent. Check your inbox!" }
                }.into_string())
            })
        )
        .route("/login", get(|
            State(state): State<AppState>,
            Extension(context): Extension<AppContext>,
            Query(params): Query<LoginParams>
        | async move {
            println!("Logging in...");
            println!("token: {}", params.token);
            // TODO: implement
            ()
        }))
}

async fn send_email(to_mailbox: Mailbox, subject: &str, content: Markup) -> Result<(), ()> {
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
        "INSERT INTO email_token (token, email_address) VALUES ($1, $2)",
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
        class="gap-4 flex flex-col items-center"
    {
        div class="flex gap-1 flex-wrap text-center justify-center" {
            p {"Log in with email to save your progress! "}
            p {(PreEscaped("&nbsp;"))"( and earn 2000xp :O )"}
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
