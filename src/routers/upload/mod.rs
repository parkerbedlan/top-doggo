use crate::{layout::base, routers::doggo::name_dog::name_dog, AppContext, AppState, FormField};
use axum::{
    body::Bytes,
    extract::{Multipart, State},
    response::Html,
    routing::get,
    Extension, Router,
};
use maud::{html, Markup, PreEscaped};
use std::{fs, path::Path};

pub fn upload_router() -> Router<AppState> {
    Router::<AppState>::new().route(
        "/",
        get(|| async move {
            base(
                upload_dog_form(FormField::empty(), FileUploadStatus::NotUploaded),
                None,
                None,
            )
        })
        .post(
            |State(state): State<AppState>,
             Extension(context): Extension<AppContext>,
             mut multipart: Multipart| async move {
                let mut dog_name: Option<String> = None;
                let mut dog_photo: Option<Bytes> = None;

                let critical_err = || Html("Error processing form".to_string());

                // extract out dog_name and dog_photo
                while let Some(field) = match multipart.next_field().await {
                    Ok(field) => field,
                    Err(error) => {
                        eprintln!("Error getting next field: {:?}", error);
                        return critical_err();
                    }
                } {
                    let name = match field.name() {
                        Some(name) => name.to_string(),
                        None => {
                            eprintln!("Field without a name");
                            return critical_err();
                        }
                    };

                    if let Some(file_type) = field.content_type() {
                        if !file_type.starts_with("image/") {
                            println!("oop {:?}", file_type);
                            return Html(
                                upload_dog_form(
                                    FormField {
                                        value: dog_name.unwrap_or("".to_string()),
                                        error: "".to_string(),
                                    },
                                    FileUploadStatus::Err("Must be an image".to_string()),
                                )
                                .into_string(),
                            );
                        } else {
                            println!("yayaya {:?}", file_type);
                        }
                    }

                    let data = match field.bytes().await {
                        Ok(data) => data,
                        Err(error) => {
                            eprintln!("Error reading bytes: {:?}", error);
                            return critical_err();
                        }
                    };

                    if name == "new_dog_name" {
                        dog_name =
                            Some(String::from_utf8(data.to_vec()).unwrap().trim().to_string());
                    } else if name == "new_dog_photo" {
                        dog_photo = Some(data);
                    }
                }
                // should always at least be an empty string
                if dog_name.is_none() {
                    eprintln!("No dog_name value");
                    return critical_err();
                }
                if dog_photo.is_none() {
                    return Html(
                        upload_dog_form(
                            FormField {
                                value: dog_name.unwrap_or("".to_string()),
                                error: "".to_string(),
                            },
                            FileUploadStatus::Err("Required".to_string()),
                        )
                        .into_string(),
                    );
                }
                let dog_name = dog_name.unwrap();
                let dog_photo = dog_photo.unwrap();

                let uploaded =
                    String::from_utf8(dog_photo.to_vec()).unwrap_or("".to_string()) == "uploaded";

                let mut transaction = state.pool.begin().await.unwrap();

                let dog_id = if uploaded {
                    let result = sqlx::query!("SELECT id FROM dog WHERE approved = FALSE AND namer_id = $1 AND name IS NULL ORDER BY id DESC LIMIT 1", context.user_id)
                        .fetch_one(&mut *transaction).await;
                    if result.is_err() {
                        eprintln!("Couldn't find uploaded dog");
                        return critical_err();
                    }
                    result.unwrap().id
                } else {
                    sqlx::query!( "INSERT INTO dog (image_url, approved, namer_id) VALUES ('temp', false, $1) RETURNING id", context.user_id)
                        .fetch_one(&mut *transaction) .await .unwrap() .id
                };

                if !uploaded {
                    let file_name = format!("{}.jpg", dog_id);
                    let file_path = Path::new("./unapproved").join(&file_name);
                    if let Err(error) = fs::write(&file_path, &dog_photo) {
                        eprintln!("Error saving file: {:?}", error);
                        // can be implicit?
                        // let _ = transaction.rollback().await;
                        return critical_err();
                    }
                    println!("Saved file '{}' to {:?}", file_name, file_path);

                    let image_url = format!("/images/{}", file_name);
                    let _ = sqlx::query!(
                        "UPDATE dog SET image_url = $1 WHERE id = $2",
                        image_url,
                        dog_id
                    )
                    // .fetch_one(&state.pool)
                    .fetch_one(&mut *transaction)
                    .await;
                }

                let _ = transaction.commit().await;

                if !dog_name.is_empty() {
                    let result = name_dog(&state.pool, context.user_id, dog_id, &dog_name).await;
                    if let Err(error) = result {
                        return Html(
                            upload_dog_form(
                                FormField {
                                    value: dog_name,
                                    error: error.to_string(),
                                },
                                FileUploadStatus::Uploaded,
                            )
                            .into_string(),
                        );
                    }
                }

                Html(html!{
                    div class="flex-1 flex flex-col gap-4 items-center justify-center text-center" {
                        h1 class="text-4xl" {"Thanks for adding your dog!"}
                        p class="text-2xl" {"Our team will approve em, and then they'll join the squad :)"}
                    }
                }.into_string())

            },
        ),
    )
}

pub enum FileUploadStatus {
    Uploaded,
    NotUploaded,
    Err(String),
}

pub fn upload_dog_form(
    new_dog_name: FormField<String>,
    file_upload_status: FileUploadStatus,
) -> Markup {
    html! {
        form
            id="upload-form"
            hx-post="/upload"
            hx-encoding="multipart/form-data"
            hx-swap="outerHTML"
            autocomplete="off"
            class="flex-1 flex flex-col items-center justify-center gap-6 max-w-sm mx-auto"
            {
            h1 class="text-5xl text-center" {"Add your dog!"}
            @match file_upload_status {
                FileUploadStatus::NotUploaded => {
                    div class="flex flex-col gap-1" {
                        input type="file" id="new_dog_photo" name="new_dog_photo" class="file-input file-input-bordered file-input-lg w-full" ;
                        label id="new_dog_photo_error" for="new_dog_photo" class="text-lg text-error leading-tight" {""}
                    }
                },
                FileUploadStatus::Uploaded => {
                    div class="border border-2 border-success bg-success bg-opacity-20 flex items-center justify-center gap-2 h-16 w-full rounded-md" {
                        div class="text-4xl" {"✅"}
                        div class="text-2xl" {"Uploaded"}
                        input type="hidden" id="new_dog_photo" name="new_dog_photo" value="uploaded" ;
                    }

                },
                FileUploadStatus::Err(error) => {
                    div class="flex flex-col gap-1" {
                        input type="file" id="new_dog_photo" name="new_dog_photo" class="file-input file-input-bordered file-input-lg w-full file-input-error" ;
                        label for="new_dog_photo" class="text-lg text-error leading-tight" {(error)}
                    }
                }
            }
            div class="flex flex-col gap-1" {
                input type="text"
                    id="new_dog_name"
                    name="new_dog_name"
                    placeholder="Name (optional)"
                    class={ "input input-bordered w-full text-lg" @if !new_dog_name.error.is_empty() {" !border-error"} }
                    value=(new_dog_name.value) ;
                label for="new_dog_name" class="text-lg text-error leading-tight" {(new_dog_name.error)}
            }
            button
                type="submit"
                class="flex flex-col justify-center items-center gap-1 bg-base-200 hover:bg-base-300 active:scale-90 transition-all duration-75 rounded-md py-4 px-8 text-2xl w-full"
                {"Add 🐕"}
            progress id="progress" value="0" max="100" class="w-full" {}
        }
        script {(PreEscaped(r#"
            htmx.on('#upload-form', 'htmx:xhr:progress', function(evt) {
                htmx.find('#progress').setAttribute('value', evt.detail.loaded/evt.detail.total * 100)
            });
            document.body.addEventListener('htmx:error', e => {
                if (e.detail.errorInfo.pathInfo.requestPath === '/upload') {
                    document.getElementById("new_dog_photo").classList.add("file-input-error");
                    document.getElementById("new_dog_photo_error").innerHTML = "That file is too big!";
                }
            });
        "#))}
    }
}
