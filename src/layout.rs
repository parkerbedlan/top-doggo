use axum::response::{Html, IntoResponse};
use maud::{html, Markup, PreEscaped, DOCTYPE};

pub fn base(content: Markup, active_nav_link_index: Option<u8>) -> impl IntoResponse {
    base_with_title_and_head(content, None, None, active_nav_link_index)
}

pub fn _base_with_title(
    content: Markup,
    title: Option<String>,
    active_nav_link_index: Option<u8>,
) -> impl IntoResponse {
    base_with_title_and_head(content, title, None, active_nav_link_index)
}

pub fn base_with_title_and_head(
    content: Markup,
    title: Option<String>,
    head: Option<Markup>,
    active_nav_link_index: Option<u8>,
) -> impl IntoResponse {
    Html(layout(content, title, head, active_nav_link_index).into_string())
}

pub fn layout(
    content: Markup,
    title: Option<String>,
    head: Option<Markup>,
    active_nav_link_index: Option<u8>,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en";
        head {
            // https://www.srihash.org/
            // https://htmx.org/docs/#installing
            script defer data-domain="doggo.parkerbedlan.com" src="https://plausible.parkerbedlan.com/js/script.js" {}
            script src="https://unpkg.com/htmx.org@2.0.0" integrity="sha384-wS5l5IKJBvK6sPTKa2WZ1js3d947pvWXbPJ1OmWfEuxLgeHcEbjUUA5i9V5ZkpCw" crossorigin="anonymous" {}
            script {(PreEscaped(r#"
                        document.addEventListener("DOMContentLoaded", () => {
                            htmx.config.useTemplateFragments = true;
                            // https://htmx.org/events/
                            // htmx.logAll() in console to see all the events as they happen!
                            document.body.addEventListener("htmx:beforeSwap", (event) => {
                                if (event.detail.xhr.status === 422) {
                                    event.detail.shouldSwap = true;
                                    // suppresses error logging in the console
                                    event.detail.isError = false;
                                }
                            })
                        })
                "#))}
            script src="https://cdn.tailwindcss.com" {}
            link rel="stylesheet" href="/output.css";
            title {(title.unwrap_or("Welcome".to_string())) " - Acme"}
            (head.unwrap_or(html!{}))
        }
        body {
            div id="content" class="max-w-screen-2xl mx-auto px-4 min-h-screen flex flex-col justify-between" hx-boost="true" {
                div {(content)}
                (navbar(active_nav_link_index))
            }
        }
    }
}

fn navbar(active_nav_link_index: Option<u8>) -> Markup {
    html! {
        footer id="navbar" class="h-16 bg-gray-200 flex justify-center items-center" {
            div class="w-full h-full flex justify-around items-center max-w-screen-lg" {
                (nav_link(html! {div class="text-2xl" {"🐶"}}, "/", if let Some(index) = active_nav_link_index {index == 0} else {false}))
                    (nav_link(html! {div class="text-9xl" {(trophy())}}, "/home", if let Some(index) = active_nav_link_index {index == 1} else {false}))
            }
        }
    }
}

fn nav_link(content: Markup, href: &str, active: bool) -> Markup {
    html! {
        a class={ "w-full h-full hover:bg-gray-300 active:scale-95 transition-all duration-75 flex items-center justify-center border-gray-800" @if active {" border-b-2"}}
        href=(href) {
            (content)
        }
    }
}

fn trophy() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6" {
            path stroke-linecap="round" stroke-linejoin="round" d="M16.5 18.75h-9m9 0a3 3 0 0 1 3 3h-15a3 3 0 0 1 3-3m9 0v-3.375c0-.621-.503-1.125-1.125-1.125h-.871M7.5 18.75v-3.375c0-.621.504-1.125 1.125-1.125h.872m5.007 0H9.497m5.007 0a7.454 7.454 0 0 1-.982-3.172M9.497 14.25a7.454 7.454 0 0 0 .981-3.172M5.25 4.236c-.982.143-1.954.317-2.916.52A6.003 6.003 0 0 0 7.73 9.728M5.25 4.236V4.5c0 2.108.966 3.99 2.48 5.228M5.25 4.236V2.721C7.456 2.41 9.71 2.25 12 2.25c2.291 0 4.545.16 6.75.47v1.516M7.73 9.728a6.726 6.726 0 0 0 2.748 1.35m8.272-6.842V4.5c0 2.108-.966 3.99-2.48 5.228m2.48-5.492a46.32 46.32 0 0 1 2.916.52 6.003 6.003 0 0 1-5.395 4.972m0 0a6.726 6.726 0 0 1-2.749 1.35m0 0a6.772 6.772 0 0 1-3.044 0" ;
        }
    }
}
