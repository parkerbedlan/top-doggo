use crate::{
    layout::{base, NavLink},
    routers::doggo::RatingType,
    AppContext, AppState,
};
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    routing::get,
    Extension, Router,
};
use maud::{html, Markup};

// in the future: /leaderboard/:superlative/:rating_type

pub fn leaderboard_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/",
            get(|| async move {
                (
                    StatusCode::TEMPORARY_REDIRECT,
                    [(header::LOCATION, "/leaderboard/top/overall")],
                )
            }),
        )
        .route(
            "/top",
            get(|| async move {
                (
                    StatusCode::TEMPORARY_REDIRECT,
                    [(header::LOCATION, "/leaderboard/top/overall")],
                )
            }),
        )
        .route(
            "/top/:rating_type",
            get(
                |State(state): State<AppState>,
                Extension(context): Extension<AppContext>,
                Path(rating_type): Path<RatingType>| async move {
                    struct LeaderboardRow {
                        value: i64,
                        name: Option<String>,
                        image_url: String
                    }
                    let ratings = match rating_type {
                        RatingType::Overall => sqlx::query_as!(
                            LeaderboardRow,
                            "select value, name, image_url from rating join dog on rating.dog_id = dog.id where type='overall' order by value desc;")
                            .fetch_all(&state.pool).await.unwrap(),
                        RatingType::Personal => sqlx::query_as!(
                            LeaderboardRow,
                            "select value, name, image_url from rating join dog on rating.dog_id = dog.id where type='personal' and user_id = $1 order by value desc;",
                            context.user_id)
                            .fetch_all(&state.pool).await.unwrap(),
                            
                    };

                    base(
                        html! {
                            div class="flex justify-center gap-4 md:gap-16 mt-4" {
                                (tab(RatingType::Overall, rating_type == RatingType::Overall))
                                (tab(RatingType::Personal, rating_type == RatingType::Personal))
                            }
                            div class="overflow-x-auto" {
                                table class="table table-sm table-zebra [&_*]:text-2xl overflow-x-auto" {
                                    thead {
                                        tr {
                                            th {}
                                            th {"Picture"}
                                            th {"Name"}
                                            th {"Rating"}
                                            // th {"Wins"}
                                            // th {"Losses"}
                                            // th {"Ties"}
                                        }
                                    }
                                    tbody {
                                        @for (i, rating) in ratings.iter().enumerate() {
                                            @let name_display = rating.name.clone().unwrap_or("A dog with no name".to_string());
                                            tr {
                                                th {(i+1)}
                                                td class="min-w-32" {img class="object-center object-cover aspect-square w-32" src=(rating.image_url) ;}
                                                td {(name_display)}
                                                td {(rating.value)}
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some("Leaderboard".to_string()),
                        Some(NavLink::Leaderboard)
                    )
                },
            ),
        )
}

fn tab(rating_type: RatingType, active: bool) -> Markup {
    let r_str = serde_json::to_string(&rating_type).unwrap();
    let r_str = &r_str[1..r_str.len() - 1];
    let capitalized = r_str[..1].to_uppercase() + &r_str[1..];
    html! {
        a class="w-48 flex justify-center active:scale-90 transition-all duration-75 text-3xl"
        href={"/leaderboard/top/"(r_str)} {
            div class={"border-purple-600" @if active {" border-b-4"}} { (capitalized) }
        }
    }
}
