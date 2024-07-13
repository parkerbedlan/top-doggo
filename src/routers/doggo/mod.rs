use crate::{layout::base, AppContext, AppState, FormField};
use axum::{
    extract::State,
    response::Html,
    routing::{get, patch},
    Extension, Form, Router,
};
use maud::{html, Markup, Render};
use serde::Deserialize;

#[derive(Debug)]
struct Dog {
    id: i64,
    image_url: String,
    name: Option<String>,
}
impl Render for Dog {
    fn render(&self) -> Markup {
        html! {
            div class="max-w-96 w-5/12 flex flex-col items-center gap-3" {
                button class="w-full aspect-square overflow-auto bg-gray-200 hover:bg-gray-300 active:scale-95 transition-all duration-75 rounded-md p-2" {
                    img class="object-center object-cover aspect-square w-full" src=(self.image_url) ;
                }
                @if let Some(name) = &self.name {
                    div class="text-3xl" {(name)}
                } @else {
                    (name_dog_form(self.id, FormField::empty()))
                }
            }

        }
    }
}

pub fn doggo_router() -> Router<AppState> {
    Router::<AppState>::new().route(
        "/",
        get(
            |State(state): State<AppState>, Extension(_context): Extension<AppContext>| async move {
                let dogs =
                    sqlx::query_as!(Dog, "SELECT id, image_url, name FROM dog LIMIT 2 OFFSET 0")
                        .fetch_all(&state.pool)
                        .await
                        .unwrap();

                base(
                    html! {
                        div class="flex flex-col items-center justify-center gap-6 flex-1" {
                            h1 class="text-5xl" {"Pick your favorite"}
                            div class="flex justify-center gap-6 w-full" {
                                (dogs[0])
                                (dogs[1])
                            }
                            div class="flex justify-center -mt-2" {
                                button class="flex flex-col justify-center items-center gap-1 bg-gray-200 hover:bg-gray-300 active:scale-90 transition-all duration-75 rounded-md aspect-square p-8" {
                                    div class="text-6xl" {"="}
                                    div class="text-lg" {"Tie"}
                                }
                            }
                        }
                    },
                    Some(0),
                )
            },
        ),
    )
    .route("/name-dog", patch(
        |State(state): State<AppState>, Extension(context): Extension<AppContext>, Form(form): Form<NameDogFormParams>| async move {
            let new_name = form.new_name.trim();

            let err = |form_error: &str| {
                Html(name_dog_form(form.dog_id, FormField {value: new_name.to_string().clone(), error: form_error.to_string()}).into_string())
            };

            if new_name == "Jeff" {
                return err("NO, don't name him Jeff >:(");
            }
            if new_name == "" {
                return err("Required");
            }
            let dog = sqlx::query!("SELECT name FROM dog WHERE id = $1", form.dog_id)
                .fetch_optional(&state.pool).await.unwrap();
            if dog.is_none() {
                return err("404: Dog not found");
            }
            let old_name = dog.unwrap().name;
            if old_name.is_some() {
                return err(&format!("{} already has a name, silly.", old_name.unwrap()));
            }
            let result = sqlx::query!("UPDATE dog SET (name, namer_id) = ($1, $2) WHERE id = $3 RETURNING name", new_name, context.user_id, form.dog_id)
                .fetch_one(&state.pool).await;
            if result.is_err() {
                return err("C'mon, something more original!");
            }
            Html(html!{div class="text-3xl" {(result.unwrap().name.unwrap())}}.into_string())
        }
    ))
}

#[derive(Deserialize, Debug)]
struct NameDogFormParams {
    dog_id: i64,
    new_name: String,
}

fn name_dog_form(dog_id: i64, new_name: FormField<String>) -> Markup {
    html! {
        form class="flex gap-1 px-4" hx-patch="/name-dog" hx-swap="outerHTML" {
            input type="hidden" name="dog_id" value=(dog_id) ;
            div class="flex flex-col" {
                input type="text" id={"new_name_"(dog_id)} name="new_name" placeholder="Name this dog!" class={ "input input-bordered w-full text-3xl" @if new_name.error != "" {" !border-error"} } value=(new_name.value) ;
                label for={"new_name_"(dog_id)} class="text-lg text-error leading-tight" {(new_name.error)}
            }
            button type="submit" class="btn text-xl" {"Submit"}
        }
    }
}
