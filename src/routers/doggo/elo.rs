use std::cmp;

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

#[derive(Clone, Copy, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RatingType {
    Overall,
    Personal,
}

/*
 * https://en.wikipedia.org/wiki/Elo_rating_system#Theory
 */
pub async fn update_ratings(
    pool: &Pool<Sqlite>,
    user_id: i64,
    dog_a_id: i64,
    dog_b_id: i64,
    rating_type: RatingType,
    status: &str,
) {
    // key for pseudocode comments: k stands for max rating change, r stands for current rating, e stands for
    // expected score, s stands for actual score, new_r stands for new rating

    // give each dog an initial rating if they don't have one yet
    // store ratings (new or old) in r_a and r_b
    let current_rating_a: u16 = get_current_rating(pool, dog_a_id, rating_type, user_id).await;
    let current_rating_b: u16 = get_current_rating(pool, dog_b_id, rating_type, user_id).await;

    // get k_a and k_b (based on how many total matches they have)
    // -1 because the current match doesn't count
    let max_rating_change_a =
        get_max_rating_change(get_num_matches(pool, dog_a_id, rating_type, user_id).await - 1);
    let max_rating_change_b =
        get_max_rating_change(get_num_matches(pool, dog_b_id, rating_type, user_id).await - 1);

    // calculate e_a and e_b as functions of r_a and r_b
    let expected_score_a: f64 = get_my_expected_score(current_rating_a, current_rating_b);
    // let expected_score_b: f64 = get_my_expected_score(current_rating_b, current_rating_a);
    let expected_score_b: f64 = 1.0 - expected_score_a;

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

    match rating_type {
        RatingType::Overall => {
            let rating_change_a = i32::from(new_rating_a) - i32::from(current_rating_a);
            let rating_change_b = i32::from(new_rating_b) - i32::from(current_rating_b);
            let _ = sqlx::query!("UPDATE match SET elo_change_overall_a = $1, elo_change_overall_b = $2 WHERE user_id = $3 AND dog_a_id = $4 AND dog_b_id = $5",
                                 rating_change_a,
                                 rating_change_b,
                                 user_id,
                                 dog_a_id,
                                 dog_b_id
                                ).fetch_one(pool).await;
        }
        RatingType::Personal => {
            let rating_change_a = i32::from(new_rating_a) - i32::from(current_rating_a);
            let rating_change_b = i32::from(new_rating_b) - i32::from(current_rating_b);
            let _ = sqlx::query!("UPDATE match SET elo_change_personal_a = $1, elo_change_personal_b = $2 WHERE user_id = $3 AND dog_a_id = $4 AND dog_b_id = $5",
                                 rating_change_a,
                                 rating_change_b,
                                 user_id,
                                 dog_a_id,
                                 dog_b_id
                                ).fetch_one(pool).await;
        }
    }

    // set the new ratings in the database
    set_rating(pool, dog_a_id, new_rating_a, rating_type, user_id).await;
    set_rating(pool, dog_b_id, new_rating_b, rating_type, user_id).await;
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

fn get_max_rating_change(num_matches: u32) -> u8 {
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
    (1.0 + 10_f64.powf((f64::from(their_current_rating) - f64::from(my_current_rating)) / 400.0))
        .powf(-1.0)
}

fn get_my_new_rating(
    my_current_rating: u16,
    my_max_rating_change: u8,
    my_actual_score: f32,
    my_expected_score: f64,
) -> u16 {
    cmp::max(
        100,
        (f64::from(my_current_rating)
            + f64::from(my_max_rating_change) * (f64::from(my_actual_score) - my_expected_score))
            .round() as u16,
    )
}

async fn set_rating(
    pool: &Pool<Sqlite>,
    dog_id: i64,
    new_rating: u16,
    rating_type: RatingType,
    user_id: i64,
) {
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
}
