use askama_axum::Template;
use axum::{
    extract::Form,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Clone, Deserialize)]
struct Contact {
    name: String,
    email: String,
}

#[derive(Template)]
#[template(path = "contacts/page.html")]
struct ContactsTemplate {
    form_data: FormTemplate,
    contacts: Vec<Contact>,
}

struct FormField<T> {
    value: T,
    error: String,
}

#[derive(Template)]
#[template(path = "contacts/form.html")]
struct FormTemplate {
    name: FormField<String>,
    email: FormField<String>,
    oob_contact: Option<Contact>,
}
fn empty_form_data() -> FormTemplate {
    FormTemplate {
        name: FormField {
            value: "".to_string(),
            error: "".to_string(),
        },
        email: FormField {
            value: "".to_string(),
            error: "".to_string(),
        },
        oob_contact: None,
    }
}

pub fn contacts_router() -> Router {
    let contacts: Arc<Mutex<Vec<Contact>>> = Arc::new(Mutex::new(vec![Contact {
        name: "Jeff".to_string(),
        email: "jeff@example.com".to_string(),
    }]));
    let contacts_1 = contacts.clone();
    let contacts_2 = contacts.clone();

    Router::new()
        .route(
            "/",
            get(|| async {
                async fn f(contacts: Arc<Mutex<Vec<Contact>>>) -> impl IntoResponse {
                    let contacts = contacts.lock().unwrap();
                    let contacts = contacts.to_vec();
                    Html(
                        ContactsTemplate {
                            contacts,
                            form_data: empty_form_data(),
                        }
                        .to_string(),
                    )
                }
                f(contacts_1).await
            }),
        )
        .route(
            "/",
            post(|Form(new_contact): Form<Contact>| async {
                async fn f(
                    contacts: Arc<Mutex<Vec<Contact>>>,
                    new_contact: Contact,
                ) -> impl IntoResponse {
                    let mut contacts = contacts.lock().unwrap();
                    contacts.push(new_contact.clone());
                    Html(
                        FormTemplate {
                            oob_contact: Some(new_contact),
                            ..empty_form_data()
                        }
                        .to_string(),
                    )
                }
                f(contacts_2, new_contact).await
            }),
        )
}
