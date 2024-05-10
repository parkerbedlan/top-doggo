use askama_axum::Template;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::routing::post;
use axum::{
    extract::Form,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Template)]
#[template(path = "counter/index.html")]
struct CounterIndexTemplate {
    count: i32,
}

#[derive(Template)]
#[template(path = "counter/count.html")]
struct CountTemplate {
    count: i32,
}

#[derive(Template)]
#[template(path = "contacts/index.html")]
struct ContactsIndexTemplate {
    contacts: Vec<Contact>,
}

#[derive(Template)]
#[template(path = "contacts/create-contact-form.html")]
struct CreateContactFormTemplate {
    name_error: String,
    email_error: String,
}

#[derive(Template, Clone)]
#[template(path = "contacts/contact.html")]
struct ContactTemplate {
    contact: Contact,
}

#[derive(Clone, Deserialize)]
struct Contact {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() {
    let count = Arc::new(Mutex::new(0));
    let count_1 = count.clone();
    let count_2 = count.clone();

    let counter_routes = Router::new()
        .route(
            "/count",
            get(|| async {
                async fn f(count: Arc<Mutex<i32>>) -> Html<String> {
                    let count = count.lock().unwrap();
                    Html(CounterIndexTemplate { count: *count }.to_string())
                }
                f(count_1).await
            }),
        )
        .route(
            "/count",
            post(|| async {
                async fn f(count: Arc<Mutex<i32>>) -> Html<String> {
                    let mut count = count.lock().unwrap();
                    *count += 1;
                    // Html(count.to_string())
                    Html(CountTemplate { count: *count }.to_string())
                }
                f(count_2).await
            }),
        );

    // let contacts = Arc::new(Mutex::new(Vec::<Contact>::new()));
    let contacts = Arc::new(Mutex::new(vec![
        Contact {
            name: "John Doe".to_string(),
            email: "johndoe@example.com".to_string(),
        },
        Contact {
            name: "Jane Doe".to_string(),
            email: "janedoe@example.com".to_string(),
        },
    ]));
    let contacts_1 = contacts.clone();
    let contacts_2 = contacts.clone();

    let contacts_routes = Router::new()
        .route(
            "/contacts",
            get(|| async {
                async fn f(contacts: Arc<Mutex<Vec<Contact>>>) -> impl IntoResponse {
                    let contacts = contacts.lock().unwrap();
                    let contacts = contacts.to_vec();
                    Html(ContactsIndexTemplate { contacts }.to_string())
                }
                f(contacts_1).await
            }),
        )
        .route(
            "/contacts",
            post(|Form(new_contact): Form<Contact>| async {
                async fn f(
                    new_contact: Contact,
                    contacts: Arc<Mutex<Vec<Contact>>>,
                ) -> impl IntoResponse {
                    let mut contacts = contacts.lock().unwrap();

                    if contacts.iter().any(|c| c.email == new_contact.email) {
                        // https://htmx.org/reference/#response_headers
                        // https://docs.rs/axum/latest/axum/response/index.html
                        // https://docs.rs/http/latest/http/header/struct.HeaderMap.html
                        let mut headers = HeaderMap::new();
                        headers.insert("HX-Reswap", HeaderValue::from_static("outerHTML"));
                        headers.insert(
                            "HX-Retarget",
                            HeaderValue::from_static("#create-contact-form"),
                        );

                        return (
                            // StatusCode::UNPROCESSABLE_ENTITY,
                            // StatusCode::OK,
                            headers,
                            // Html("Email already exists".to_string()),
                            Html(
                                CreateContactFormTemplate {
                                    name_error: "".to_string(),
                                    email_error: "Email already exists".to_string(),
                                }
                                .to_string(),
                            ),
                        );
                    }

                    contacts.push(new_contact.clone());

                    (
                        // StatusCode::OK,
                        HeaderMap::new(),
                        Html(
                            ContactTemplate {
                                contact: new_contact,
                            }
                            .to_string(),
                        ),
                    )
                }
                f(new_contact, contacts_2).await
            }),
        );

    // https://docs.rs/axum/latest/axum/routing/struct.Router.html#method.nest
    let app = Router::new()
        .nest("/", counter_routes)
        .nest("/", contacts_routes)
        .nest_service("/", ServeDir::new("assets"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await
        .unwrap();
}
