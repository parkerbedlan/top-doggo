use crate::{AppContext, AppState, FormField};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    Extension, Form,
};
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};

pub async fn name_dog_router(
    State(state): State<AppState>,
    Extension(context): Extension<AppContext>,
    Form(form): Form<NameDogFormParams>,
) -> impl IntoResponse {
    let new_name = form.new_name.trim();

    let err = |form_error: &str| {
        Html(
            name_dog_form(
                form.dog_id,
                FormField {
                    value: new_name.to_string().clone(),
                    error: form_error.to_string(),
                },
            )
            .into_string(),
        )
    };

    if new_name.is_empty() {
        return err("^ Type this dog's new name right up here :)");
    }

    let result = name_dog(&state.pool, context.user_id, form.dog_id, new_name).await;
    if let Err(error) = result {
        return err(&error);
    }

    let client_ip: Option<String> = if let Some(ip) = context.client_ip {
        Some(ip.to_string())
    } else {
        None
    };
    let _ = sqlx::query!(
        "INSERT INTO log (action, user_id, client_ip, notes) VALUES ('name-dog', $1, $2, $3)",
        context.user_id,
        client_ip,
        form.dog_id
    )
    .fetch_one(&state.pool)
    .await;

    Html(html! {div class="text-3xl" {(result.unwrap())}}.into_string())
}

pub async fn name_dog(
    pool: &Pool<Sqlite>,
    user_id: i64,
    dog_id: i64,
    new_name: &str,
) -> Result<String, String> {
    if new_name == "Jeff" {
        return Err("NO, don't name him Jeff >:(".to_string());
    }
    let dog = sqlx::query!("SELECT name FROM dog WHERE id = $1", dog_id)
        .fetch_optional(pool)
        .await
        .unwrap();
    if dog.is_none() {
        return Err("404: Dog not found".to_string());
    }
    let old_name = dog.unwrap().name;
    if old_name.is_some() {
        return Err(format!("{} already has a name, silly.", old_name.unwrap()));
    }

    let result = sqlx::query!(
        "UPDATE dog SET (name, namer_id) = ($1, $2) WHERE id = $3 RETURNING name",
        new_name,
        user_id,
        dog_id
    )
    .fetch_one(pool)
    .await;
    if result.is_err() {
        return Err("C'mon, something more original!".to_string());
    }
    Ok(result.unwrap().name.unwrap())
}

#[derive(Deserialize, Debug)]
pub struct NameDogFormParams {
    dog_id: i64,
    new_name: String,
}

pub fn name_dog_form(dog_id: i64, new_name: FormField<String>) -> Markup {
    html! {
        // https://gist.github.com/niksumeiko/360164708c3b326bd1c8?permalink_comment_id=3925346#gistcomment-3925346
        form class="flex gap-1 flex-wrap sm:flex-nowrap" hx-patch="/name-dog" hx-swap="outerHTML" autocomplete="off" {
            input type="hidden" name="dog_id" value=(dog_id) autocomplete="off" ;
            div class="flex flex-col w-full max-w-64 flex-shrink-0" {
                input type="text"
                    id={"new_name_"(dog_id)}
                    name="new_name"
                    placeholder="Name this dog!"
                    class={ "input input-bordered w-full text-lg px-2" @if !new_name.error.is_empty() {" !border-error"} }
                    value=(new_name.value) ;
                label for={"new_name_"(dog_id)} class="text-lg text-error leading-tight" {(new_name.error)}
            }
            button type="submit"
                class="flex flex-col justify-center items-center gap-1 bg-base-200 hover:bg-base-300 active:scale-90 transition-all duration-75 rounded-md p-2 w-full flex-shrink"
                {(tag_icon())}
        }
    }
}

pub fn tag_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6" {
            path stroke-linecap="round" stroke-linejoin="round" d="M9.568 3H5.25A2.25 2.25 0 0 0 3 5.25v4.318c0 .597.237 1.17.659 1.591l9.581 9.581c.699.699 1.78.872 2.607.33a18.095 18.095 0 0 0 5.223-5.223c.542-.827.369-1.908-.33-2.607L11.16 3.66A2.25 2.25 0 0 0 9.568 3Z" ;
            path stroke-linecap="round" stroke-linejoin="round" d="M6 6h.008v.008H6V6Z" ;
        }
    }
}
