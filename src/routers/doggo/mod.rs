use self::elo::RatingType;
use crate::{layout::base, AppContext, AppState, FormField};
use axum::{
    extract::{Path, State},
    response::Html,
    routing::{get, patch, post},
    Extension, Form, Router,
};
use maud::{html, Markup, Render};
use rand::seq::SliceRandom;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};

mod elo;

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
                button hx-post={"/pick-winner/"(self.id)} hx-target="#game-board" hx-swap="outerHTML" class="w-full aspect-square overflow-auto bg-gray-200 hover:bg-gray-300 active:scale-95 transition-all duration-75 rounded-md p-2" {
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

async fn get_dog(dog_id: i64, pool: &Pool<Sqlite>) -> Option<Dog> {
    let result = sqlx::query_as!(
        Dog,
        "SELECT id, image_url, name FROM dog WHERE id = $1",
        dog_id
    )
    .fetch_one(pool)
    .await;
    match result {
        Ok(dog) => Some(dog),
        Err(_) => None,
    }
}

struct DogMatch {
    id: i64,
    dog_a_id: i64,
    dog_b_id: i64,
}
async fn get_current_dog_match(user_id: i64, pool: &Pool<Sqlite>) -> Option<DogMatch> {
    sqlx::query_as!(
        DogMatch,
        "SELECT id, dog_a_id, dog_b_id FROM match WHERE user_id=$1 AND status='…' LIMIT 1",
        user_id
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None)
}

async fn get_dog_match(user_id: i64, pool: &Pool<Sqlite>) -> Option<(Dog, Dog)> {
    let current_dog_match = get_current_dog_match(user_id, pool).await;
    if let Some(dog_match) = current_dog_match {
        let dog_a = get_dog(dog_match.dog_a_id, pool).await.unwrap();
        let dog_b = get_dog(dog_match.dog_b_id, pool).await.unwrap();
        return Some((dog_a, dog_b));
    }

    let valid_dog_ids = sqlx::query!(
        "SELECT id FROM dog WHERE id NOT IN (SELECT dog_id AS id FROM user_finished_with_dog WHERE user_id=$1)", 
        user_id)
        .fetch_all(pool).await.unwrap();

    if valid_dog_ids.is_empty() {
        return None;
    }

    let dog_a_id = valid_dog_ids.choose(&mut rand::thread_rng()).unwrap().id;

    let potential_dog_b_ids = sqlx::query!(
        "SELECT id FROM dog WHERE id <> $1 AND id NOT IN (SELECT dog_a_id AS id FROM match WHERE dog_b_id=$1 AND user_id=$2 UNION SELECT dog_b_id AS id FROM match WHERE dog_a_id=$1 AND user_id=$2)",
        dog_a_id, user_id)
        .fetch_all(pool).await.unwrap();

    if potential_dog_b_ids.is_empty() {
        let _ = sqlx::query!(
            "INSERT INTO user_finished_with_dog (user_id, dog_id) VALUES ($1, $2)",
            user_id,
            dog_a_id
        )
        .fetch_one(pool)
        .await;

        return Box::pin(get_dog_match(user_id, pool)).await;
    }

    let dog_b_id = potential_dog_b_ids
        .choose(&mut rand::thread_rng())
        .unwrap()
        .id;

    let _ = sqlx::query!(
        "INSERT INTO match (user_id, dog_a_id, dog_b_id) VALUES ($1, $2, $3)",
        user_id,
        dog_a_id,
        dog_b_id
    )
    .fetch_one(pool)
    .await;

    let dog_a = get_dog(dog_a_id, pool).await.unwrap();
    let dog_b = get_dog(dog_b_id, pool).await.unwrap();
    Some((dog_a, dog_b))
}

fn game_board(dogs: Option<(Dog, Dog)>) -> Markup {
    let Some((dog_a, dog_b)) = dogs else {
        return html! {
            div class="flex flex-col items-center justify-center gap-6 flex-1" {
                h1 class="text-5xl" {"You've won!"}
                p {"(Now please go outside and touch grass and pet a real dog or something)"}
            }
        };
    };
    html! {
        div id="game-board" class="flex flex-col items-center justify-center gap-6 flex-1" {
            h1 class="text-5xl" {"Pick your favorite"}
            div class="flex justify-center gap-6 w-full" {
                (dog_a)
                (dog_b)
            }
            div class="flex justify-center -mt-2" {
                button hx-post="/pick-winner/tie" hx-target="#game-board" hx-swap="outerHTML" class="flex flex-col justify-center items-center gap-1 bg-gray-200 hover:bg-gray-300 active:scale-90 transition-all duration-75 rounded-md aspect-square p-8" {
                    div class="text-6xl" {"="}
                    div class="text-lg" {"Tie"}
                }
            }
        }
    }
}

pub fn doggo_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/", get(
            |State(state): State<AppState>, Extension(context): Extension<AppContext>| async move {
                let dogs = get_dog_match(context.user_id, &state.pool).await;
                base(
                    html! {
                        (game_board(dogs))
                    },
                    Some(0),
                )
            },
        ))
        .route("/name-dog", patch(
            |State(state): State<AppState>, Extension(context): Extension<AppContext>, Form(form): Form<NameDogFormParams>| async move {
                let new_name = form.new_name.trim();

                let err = |form_error: &str| {
                    Html(name_dog_form(form.dog_id, FormField {value: new_name.to_string().clone(), error: form_error.to_string()}).into_string())
                };

                if new_name == "Jeff" {
                    return err("NO, don't name him Jeff >:(");
                }
                if new_name.is_empty() {
                    return err("^ Type this dog's new name right up here :)");
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
        .route("/pick-winner/:winner", post(
            |State(state): State<AppState>, Extension(context): Extension<AppContext>, Path(winner): Path<String>| async move {
                let pool = &state.pool;
                let user_id = context.user_id;

                let new_game_board = || async {
                    let new_dog_match = get_dog_match(user_id, pool).await;
                    Html(game_board(new_dog_match).into_string())
                };

                let current_dog_match = get_current_dog_match(user_id, pool).await;
                if current_dog_match.is_none() {
                    return new_game_board().await;
                }
                let current_dog_match = current_dog_match.unwrap();

                let status: Result<&str, ()> = if winner == current_dog_match.dog_a_id.to_string() {Ok(">")}
                    else if winner == current_dog_match.dog_b_id.to_string() {Ok("<")}
                    else if winner == "tie" {Ok("=")}
                    else {Err(())};
                if status.is_err() {
                    return new_game_board().await;
                }
                let status = status.unwrap();

                let _ = sqlx::query!("UPDATE match SET status = $1 WHERE id = $2", status, current_dog_match.id).fetch_one(pool).await;

                let DogMatch {dog_a_id, dog_b_id, ..} = current_dog_match;

                elo::update_ratings(pool, user_id, dog_a_id, dog_b_id, RatingType::Overall, status).await;
                elo::update_ratings(pool, user_id, dog_a_id, dog_b_id, RatingType::Personal, status).await;

                new_game_board().await
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
        // https://gist.github.com/niksumeiko/360164708c3b326bd1c8?permalink_comment_id=3925346#gistcomment-3925346
        form class="flex gap-1 px-4" hx-patch="/name-dog" hx-swap="outerHTML" autocomplete="off" {
            input type="hidden" name="dog_id" value=(dog_id) autocomplete="off" ;
            div class="flex flex-col" {
                input type="text" id={"new_name_"(dog_id)} name="new_name" placeholder="Name this dog!" class={ "input input-bordered w-full text-3xl" @if !new_name.error.is_empty() {" !border-error"} } value=(new_name.value) ;
                label for={"new_name_"(dog_id)} class="text-lg text-error leading-tight" {(new_name.error)}
            }
            button type="submit" class="btn text-xl" {(tag_icon())}
        }
    }
}

fn tag_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6" {
            path stroke-linecap="round" stroke-linejoin="round" d="M9.568 3H5.25A2.25 2.25 0 0 0 3 5.25v4.318c0 .597.237 1.17.659 1.591l9.581 9.581c.699.699 1.78.872 2.607.33a18.095 18.095 0 0 0 5.223-5.223c.542-.827.369-1.908-.33-2.607L11.16 3.66A2.25 2.25 0 0 0 9.568 3Z" ;
            path stroke-linecap="round" stroke-linejoin="round" d="M6 6h.008v.008H6V6Z" ;
        }
    }
}
