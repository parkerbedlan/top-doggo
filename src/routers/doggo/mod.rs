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

    if valid_dog_ids.len() == 0 {
        return None;
    }

    let dog_a_id = valid_dog_ids.choose(&mut rand::thread_rng()).unwrap().id;

    let potential_dog_b_ids = sqlx::query!(
        "SELECT id FROM dog WHERE id <> $1 AND id NOT IN (SELECT dog_a_id AS id FROM match WHERE dog_b_id=$1 AND user_id=$2 UNION SELECT dog_b_id AS id FROM match WHERE dog_a_id=$1 AND user_id=$2)",
        dog_a_id, user_id)
        .fetch_all(pool).await.unwrap();

    if potential_dog_b_ids.len() == 0 {
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
                if new_name == "" {
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

                update_ratings(pool, user_id, dog_a_id, dog_b_id, RatingType::Overall, status).await;
                update_ratings(pool, user_id, dog_a_id, dog_b_id, RatingType::Personal, status).await;

                return new_game_board().await;
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
                input type="text" id={"new_name_"(dog_id)} name="new_name" placeholder="Name this dog!" class={ "input input-bordered w-full text-3xl" @if new_name.error != "" {" !border-error"} } value=(new_name.value) ;
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

#[derive(Clone, Copy)]
enum RatingType {
    Overall,
    Personal,
}
async fn get_current_rating(
    pool: &Pool<Sqlite>,
    dog_id: i64,
    rating_type: RatingType,
    user_id: i64,
) -> u16 {
    struct RatingValueResult {
        value: i64,
    }

    let mut current_rating = match rating_type {
        RatingType::Overall => sqlx::query_as!(
            RatingValueResult,
            "SELECT value FROM rating WHERE dog_id=$1 AND type='overall'",
            dog_id
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None),
        // TODO: determine whether sqlx properly coerces user_id from Option<i64> to i64 for this query
        RatingType::Personal => sqlx::query_as!(
            RatingValueResult,
            "SELECT value FROM rating WHERE dog_id=$1 AND type='personal' AND user_id=$2",
            dog_id,
            user_id
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None),
    };
    if current_rating.is_none() {
        current_rating = Some(match rating_type {
            RatingType::Overall => sqlx::query_as!(
                RatingValueResult,
                "INSERT INTO rating (dog_id) VALUES ($1) RETURNING value",
                dog_id
            ).fetch_one(pool).await.unwrap(),
            RatingType::Personal => sqlx::query_as!(
                RatingValueResult,
                "INSERT INTO rating (dog_id, type, user_id) VALUES ($1, 'personal', $2) RETURNING value",
                dog_id, user_id
            ).fetch_one(pool).await.unwrap()
        });
    }
    let current_rating: u16 = current_rating.unwrap().value.try_into().unwrap();
    current_rating
}

// TODO: rename to get_max_rating_change
fn get_k(num_matches: u32) -> u8 {
    if num_matches < 5 {
        128
    } else if num_matches < 10 {
        64
    } else {
        32
    }
}
async fn get_num_matches(
    pool: &Pool<Sqlite>,
    dog_id: i64,
    rating_type: RatingType,
    user_id: i64,
) -> u32 {
    match rating_type {
        RatingType::Overall => sqlx::query!(
            "SELECT COUNT(*) as count FROM match WHERE dog_a_id=$1 OR dog_b_id=$1",
            dog_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .count
        .try_into()
        .unwrap(),
        RatingType::Personal => sqlx::query!(
            "SELECT COUNT(*) as count FROM match WHERE dog_a_id=$1 OR dog_b_id=$1 AND user_id=$2",
            dog_id,
            user_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .count
        .try_into()
        .unwrap(),
    }
}

fn get_my_expected_score(my_current_rating: u16, their_current_rating: u16) -> f64 {
    (1.0 + 10_f64.powf(f64::from(their_current_rating - my_current_rating) / 400.0)).powf(-1.0)
}

fn get_my_new_rating(
    my_current_rating: u16,
    my_max_rating_change: u8,
    my_actual_score: f32,
    my_expected_score: f64,
) -> u16 {
    (f64::from(my_current_rating)
        + f64::from(my_max_rating_change) * (f64::from(my_actual_score) - my_expected_score))
        .round() as u16
}

async fn set_rating(
    pool: &Pool<Sqlite>,
    dog_id: i64,
    new_rating: u16,
    rating_type: RatingType,
    user_id: i64,
) -> () {
    match rating_type {
        RatingType::Overall => {
            let _ = sqlx::query!(
                "UPDATE rating SET value = $1 WHERE dog_id = $2 AND type = 'overall'",
                new_rating,
                dog_id
            )
            .fetch_one(pool)
            .await;
        }
        RatingType::Personal => {
            let _ = sqlx::query!("UPDATE rating SET value = $1 WHERE dog_id = $2 AND type = 'personal' AND user_id = $3", new_rating, dog_id, user_id).fetch_one(pool).await;
        }
    };
    ()
}

async fn update_ratings(
    pool: &Pool<Sqlite>,
    user_id: i64,
    dog_a_id: i64,
    dog_b_id: i64,
    rating_type: RatingType,
    status: &str,
) -> () {
    // key for pseudocode: k stands for max rating change, r stands for current rating, e stands for
    // expected score, s stands for actual score, new_r stands for new rating

    // give each dog an initial rating if they don't have one yet
    // store ratings (new or old) in r_a and r_b
    let current_rating_a: u16 = get_current_rating(pool, dog_a_id, rating_type, user_id).await;
    let current_rating_b: u16 = get_current_rating(pool, dog_b_id, rating_type, user_id).await;

    // get k_a and k_b (based on how many total matches they have)
    // -1 because the current match doesn't count
    let max_rating_change_a =
        get_k(get_num_matches(pool, dog_a_id, rating_type, user_id).await - 1);
    let max_rating_change_b =
        get_k(get_num_matches(pool, dog_b_id, rating_type, user_id).await - 1);

    // calculate e_a and e_b as functions of r_a and r_b
    let expected_score_a: f64 = get_my_expected_score(current_rating_a, current_rating_b);
    let expected_score_b: f64 = get_my_expected_score(current_rating_b, current_rating_a);

    // set s_a and s_b as functions of status (s_b is just 1 - s_a)
    let actual_score_a: f32 = if status == ">" {
        1.0
    } else if status == "<" {
        0.0
    } else if status == "=" {
        0.5
    } else {
        -10000.0
    };
    let actual_score_b: f32 = 1.0 - actual_score_a;

    // calculate new_r_a as a function of r_a, k_a, s_a, and e_a
    // same for b
    let new_rating_a = get_my_new_rating(
        current_rating_a,
        max_rating_change_a,
        actual_score_a,
        expected_score_a,
    );
    let new_rating_b = get_my_new_rating(
        current_rating_b,
        max_rating_change_b,
        actual_score_b,
        expected_score_b,
    );

    // set the new ratings in the database
    set_rating(pool, dog_a_id, new_rating_a, rating_type, user_id).await;
    set_rating(pool, dog_b_id, new_rating_b, rating_type, user_id).await;

    ()
}
