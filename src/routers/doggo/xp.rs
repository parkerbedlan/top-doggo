use maud::{html, Markup, PreEscaped};
use rand::Rng;
use sqlx::{Pool, Sqlite};

pub fn get_xp_threshold(level: u32) -> u32 {
    1000 * (level) * (level + 1) / 2
}

pub fn get_level(xp: u32) -> u32 {
    (f64::sqrt((1 + 8 * (xp / 1000)) as f64) as u32 - 1) / 2
}

pub fn get_xp_remainder(xp: u32) -> u32 {
    let level = get_level(xp);
    let last_threshold = get_xp_threshold(level);
    xp - last_threshold
}

pub fn get_next_xp_target(xp: u32) -> u32 {
    let level = get_level(xp);
    (level + 1) * 1000
}

pub fn get_xp_increase_from_pick(seconds_deliberated: u32) -> u32 {
    if seconds_deliberated < 2 {
        1
    } else {
        let lower_bound = seconds_deliberated * 6;
        let upper_bound = seconds_deliberated * 14;
        rand::thread_rng().gen_range(lower_bound..=upper_bound)
    }
}

pub fn xp_section(xp: u32, xp_increase: Option<u32>, oob: bool) -> Markup {
    html! {
            section id="xp-section" hx-swap-oob=[if oob {Some("outerHTML")} else {None}] class="flex flex-col gap-3 items-center w-full max-w-screen-sm px-2 relative" {
                h3 class="text-2xl text-center" {"Level "(get_level(xp))}
                div class="w-full flex items-center justify-center gap-3" {
                    div class="w-1/6 text-right" {(get_xp_remainder(xp))(PreEscaped("&nbsp;"))"xp"}
                    div id="xp-bar-wrapper" class="w-1/2 h-5 rounded-full bg-base-200 overflow-hidden" {
                        div id="xp-bar" class="w-full h-full rounded-full bg-purple-400 transition-all duration-1000"
                        style={"transform: translateX(-"((1.0 - (get_xp_remainder(xp) as f64 / get_next_xp_target(xp) as f64))*100.0)"%);"}
                        {}
                    }
                    div class="w-1/6" {(get_next_xp_target(xp))(PreEscaped("&nbsp;"))"xp"}
                }
                @if let Some(inc) = xp_increase {
                    div class="absolute -bottom-6 -left-50 -right-50 mx-auto animate-scale-up-down" {"+"(inc)" xp"}
                }
            }
    }
}

pub async fn get_xp(pool: &Pool<Sqlite>, user_id: i64) -> u32 {
    sqlx::query!("SELECT total_xp FROM user WHERE id = $1", user_id)
        .fetch_one(pool)
        .await
        .unwrap()
        .total_xp as u32
}

pub const XP_INCREASE_FOR_NAME_DOG: u32 = 200;
