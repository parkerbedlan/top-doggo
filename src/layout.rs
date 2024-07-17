use axum::response::{Html, IntoResponse};
use maud::{html, Markup, PreEscaped, DOCTYPE};

pub fn base(
    content: Markup,
    title: Option<String>,
    // head: Option<Markup>,
    active_nav_link: Option<NavLink>,
) -> impl IntoResponse {
    Html(layout(content, title, active_nav_link).into_string())
}

pub fn layout(
    content: Markup,
    title: Option<String>,
    // head: Option<Markup>,
    active_nav_link: Option<NavLink>,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en";
        head {
            // FOR PROD uncomment the Plausible analytics
            script defer data-domain="doggo.parkerbedlan.com" src="https://plausible.parkerbedlan.com/js/script.js" {}
            // https://www.srihash.org/
            // https://htmx.org/docs/#installing
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
            // FOR PROD comment out the tailwind cdn
            // script src="https://cdn.tailwindcss.com" {}
            link rel="stylesheet" href="/output.css";
            title {@if title.is_none() {"Top Doggo"} @else {(&format!("{} - Top Doggo", title.unwrap()))}}
            meta name="HandheldFriendly" content="true" ;
            meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" ;
            // (head.unwrap_or(html!{}))
        }
        body class="max-w-screen-2xl mx-auto pb-16 min-h-[100dvh] flex flex-col font-shantell overflow-x-hidden" hx-boost="true" {
            {(content)}
            (navbar(active_nav_link))
        }

    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum NavLink {
    Root,
    Leaderboard,
    // Me
}

fn navbar(active_nav_link: Option<NavLink>) -> Markup {
    html! {
        footer id="navbar" class="fixed bottom-0 left-0 right-0 h-16 bg-base-200 flex justify-center items-center border-base-100 border-t" {
            div class="w-full h-full flex justify-around items-center max-w-screen-lg" {
                (nav_link(html! {div class="text-2xl" {"🐶"}}, "/", if let Some(link) = active_nav_link {link == NavLink::Root} else {false}))
                (nav_link(html! {div class="text-9xl text-yellow-500" {(trophy_icon())}}, "/leaderboard", if let Some(link) = active_nav_link {link == NavLink::Leaderboard} else {false}))
            }
        }
    }
}

fn nav_link(content: Markup, href: &str, active: bool) -> Markup {
    html! {
        a class={ "w-full h-full hover:bg-base-300 active:scale-90 transition-all duration-75 flex items-center justify-center border-purple-600" @if active {" border-b-4"}}
        href=(href)
        style={@if active {""} @else {"-webkit-filter: grayscale(1)"}} {
            (content)
        }
    }
}

fn trophy_icon() -> Markup {
    html! {
            svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="size-6" {
                path fill-rule="evenodd" d="M5.166 2.621v.858c-1.035.148-2.059.33-3.071.543a.75.75 0 0 0-.584.859 6.753 6.753 0 0 0 6.138 5.6 6.73 6.73 0 0 0 2.743 1.346A6.707 6.707 0 0 1 9.279 15H8.54c-1.036 0-1.875.84-1.875 1.875V19.5h-.75a2.25 2.25 0 0 0-2.25 2.25c0 .414.336.75.75.75h15a.75.75 0 0 0 .75-.75 2.25 2.25 0 0 0-2.25-2.25h-.75v-2.625c0-1.036-.84-1.875-1.875-1.875h-.739a6.706 6.706 0 0 1-1.112-3.173 6.73 6.73 0 0 0 2.743-1.347 6.753 6.753 0 0 0 6.139-5.6.75.75 0 0 0-.585-.858 47.077 47.077 0 0 0-3.07-.543V2.62a.75.75 0 0 0-.658-.744 49.22 49.22 0 0 0-6.093-.377c-2.063 0-4.096.128-6.093.377a.75.75 0 0 0-.657.744Zm0 2.629c0 1.196.312 2.32.857 3.294A5.266 5.266 0 0 1 3.16 5.337a45.6 45.6 0 0 1 2.006-.343v.256Zm13.5 0v-.256c.674.1 1.343.214 2.006.343a5.265 5.265 0 0 1-2.863 3.207 6.72 6.72 0 0 0 .857-3.294Z" clip-rule="evenodd" ;
        }
    }
}
