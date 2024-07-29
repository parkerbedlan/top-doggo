pub use self::elo::RatingType;
use crate::{
    layout::{base, NavLink},
    routers::doggo::xp::{get_xp, get_xp_increase_from_pick, xp_section},
    AppContext, AppState, FormField,
};
use axum::{
    extract::{Path, State},
    response::Html,
    routing::{get, patch, post},
    Extension, Router,
};
use maud::{html, Markup, Render};
use rand::seq::SliceRandom;
use sqlx::{Pool, Sqlite};
use std::cmp;

mod elo;
pub mod name_dog;
mod xp;

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
                button hx-post={"/pick-winner/"(self.id)} hx-target="#game-board" hx-swap="outerHTML transition:true" class="w-full aspect-square overflow-auto bg-base-200 hover:bg-base-300 active:scale-95 transition-all duration-75 rounded-md p-2" {
                    img class="object-center object-cover aspect-square w-full" src=(self.image_url) ;
                }
                @if let Some(name) = &self.name {
                    div class="text-3xl break-words max-w-full" {(name)}
                } @else {
                    (name_dog::name_dog_form(self.id, FormField::empty()))
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
        "SELECT id FROM dog WHERE approved = TRUE AND id NOT IN (SELECT dog_id AS id FROM user_finished_with_dog WHERE user_id=$1)", 
        user_id)
        .fetch_all(pool).await.unwrap();

    if valid_dog_ids.is_empty() {
        return None;
    }

    let dog_a_id = valid_dog_ids.choose(&mut rand::thread_rng()).unwrap().id;

    let potential_dog_b_ids = sqlx::query!(
        "SELECT id FROM dog WHERE approved = TRUE AND id <> $1 AND id NOT IN (SELECT dog_a_id AS id FROM match WHERE dog_b_id=$1 AND user_id=$2 UNION SELECT dog_b_id AS id FROM match WHERE dog_a_id=$1 AND user_id=$2)",
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

async fn game_board(user_id: i64, pool: &Pool<Sqlite>, xp_increase: Option<u32>) -> Markup {
    let dogs = get_dog_match(user_id, pool).await;
    let Some((dog_a, dog_b)) = dogs else {
        return html! {
            div class="flex flex-col items-center justify-center gap-6 flex-1" {
                h1 class="text-5xl" {"You've won! Check out " a href="/leaderboard" class="underline text-blue-700" {"the leaderboard!"}}
                p {"(Then please go outside and touch grass and pet a real dog or something)"}
            }
        };
    };

    let xp = get_xp(pool, user_id).await;

    html! {
        div id="game-board" class="flex flex-col items-center justify-center gap-6 flex-1" {
            (xp_section(xp, xp_increase, false))
            h1 class="text-5xl text-center" {"Pick your favorite"}
            div class="flex justify-center gap-6 w-full vt-slide-up" {
                (dog_a)
                (dog_b)
            }
            div class="flex justify-center -mt-2" {
                button hx-post="/pick-winner/tie" hx-target="#game-board" hx-swap="outerHTML transition:true" class="flex flex-col justify-center items-center gap-1 bg-base-200 hover:bg-base-300 active:scale-90 transition-all duration-75 rounded-md w-28 h-28 p-8 vt-stay-on-top" {
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
                println!("ip {:?}", context.client_ip);
                base(
                    html! {
                        (game_board(context.user_id, &state.pool, None).await)
                    },
                    None,
                    Some(NavLink::Root)
                )
            },
        ))
        .route("/name-dog", patch(name_dog::name_dog_router))
        .route("/pick-winner/:winner", post(
            |State(state): State<AppState>, Extension(context): Extension<AppContext>, Path(winner): Path<String>| async move {
                let pool = &state.pool;
                let user_id = context.user_id;

                let new_game_board = |xp_increase: Option<u32>| async move {
                    Html(game_board(user_id, pool, xp_increase).await.into_string())
                };

                let current_dog_match = get_current_dog_match(user_id, pool).await;
                if current_dog_match.is_none() {
                    return new_game_board(None).await;
                }
                let current_dog_match = current_dog_match.unwrap();

                let status: Result<&str, ()> = if winner == current_dog_match.dog_a_id.to_string() {Ok(">")}
                    else if winner == current_dog_match.dog_b_id.to_string() {Ok("<")}
                    else if winner == "tie" {Ok("=")}
                    else {Err(())};
                if status.is_err() {
                    return new_game_board(None).await;
                }
                let status = status.unwrap();

                let _ = sqlx::query!("UPDATE match SET status = $1 WHERE id = $2", status, current_dog_match.id).fetch_one(pool).await;

                let DogMatch {dog_a_id, dog_b_id, ..} = current_dog_match;

                elo::update_ratings(pool, user_id, dog_a_id, dog_b_id, RatingType::Overall, status).await;
                elo::update_ratings(pool, user_id, dog_a_id, dog_b_id, RatingType::Personal, status).await;

                struct SecondsRecord {seconds: Option<i64>}
                let seconds_deliberated = sqlx::query_as!(SecondsRecord, "select CAST ((JulianDay(updated_at) - JulianDay(created_at)) * 24 * 60 * 60 AS INTEGER) as seconds FROM match WHERE id = $1 AND created_at IS NOT NULL AND updated_at IS NOT NULL", current_dog_match.id).fetch_one(pool).await.unwrap_or(SecondsRecord { seconds: Some(5) }).seconds.unwrap();
                let seconds_deliberated = cmp::min(5, seconds_deliberated) as u32;

                let xp_increase: u32 = get_xp_increase_from_pick(seconds_deliberated);

                let _ = sqlx::query!("UPDATE user SET total_xp = total_xp + $1 WHERE id = $2", xp_increase, user_id).fetch_one(pool).await;

                new_game_board(Some(xp_increase)).await
            }
        ))
        .route("/dedication", get(|| async move {
            base(html! {
                div class="flex-1 flex flex-col gap-4 items-center justify-center text-center" {
                    h1 class="text-4xl" {"This app is dedicated to Chef Alex,"}
                    p class="text-3xl" {"the biggest dog fan I know."}
                    p class="text-2xl" {"🐕 🐩 🐶 🐕‍🦺 🦮"}
                }
            }, Some("Dedication".to_string()), None)
        }))
}
