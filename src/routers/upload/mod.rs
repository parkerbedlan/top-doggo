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
            let new_dog_name: FormField<String> = FormField::empty();
            // let new_dog_name: FormField<String> = FormField {value: "".to_string(), error: "Uh oh".to_string()};
            base(
                upload_dog_form(new_dog_name, FileUploadStatus::NotUploaded),
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
                        // TODO: validate file
                        dog_photo = Some(data);
                    }
                }
                if dog_name.is_none() || dog_photo.is_none() {
                    return critical_err();
                }
                let dog_name = dog_name.unwrap();
                let dog_photo = dog_photo.unwrap();

                // create the unapproved dog, returning the id
                let dog_id = sqlx::query!(
                    "INSERT INTO dog (image_url, approved) VALUES ('temp', false) RETURNING id",
                )
                .fetch_one(&state.pool)
                .await
                .unwrap()
                .id;

                // upload the photo with the id as a name
                let file_name = format!("{}.jpg", dog_id);
                let file_path = Path::new("./unapproved").join(&file_name);
                if let Err(error) = fs::write(&file_path, &dog_photo) {
                    eprintln!("Error saving file: {:?}", error);
                    return critical_err();
                }
                println!("Saved file '{}' to {:?}", file_name, file_path);

                // add the image_url
                let image_url = format!("/images/{}", file_name);
                let _ = sqlx::query!(
                    "UPDATE dog SET image_url = $1 WHERE id = $2",
                    image_url,
                    dog_id
                )
                .fetch_one(&state.pool)
                .await;

                // use the name_dog function
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

                Html(
                    upload_dog_form(FormField::empty(), FileUploadStatus::NotUploaded)
                        .into_string(),
                )
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
                    input type="file" id="new_dog_photo" name="new_dog_photo" class="file-input file-input-bordered file-input-lg w-full" ;
                },
                FileUploadStatus::Uploaded => {
                    div class="border border-2 border-success bg-success bg-opacity-20 flex items-center justify-center" {
                        div {"✔️"}
                        div {"Uploeaded"}
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
        "#))}
    }
}
