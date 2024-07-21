use std::{fs, path::Path};

use crate::{layout::base, AppState, FormField};
use axum::{extract::Multipart, response::Html, routing::get, Router};
use maud::{html, Markup, PreEscaped};

pub fn upload_router() -> Router<AppState> {
    Router::<AppState>::new().route(
        "/",
        get(|| async move {
            let new_dog_name: FormField<String> = FormField::empty();
            // let new_dog_name: FormField<String> = FormField {value: "".to_string(), error: "Uh oh".to_string()};
            base(upload_dog_form(new_dog_name), None, None)
        })
        .post(|mut multipart: Multipart| async move {
            let err = || Html("Error processing form".to_string());

            // should already exist as docker volume, therefore unnecessary
            // // Ensure the ./unapproved directory exists
            // if let Err(error) = fs::create_dir_all("./unapproved") {
            //     eprintln!("Error creating directory: {:?}", error);
            //     return err();
            // }

            while let Some(field) = match multipart.next_field().await {
                Ok(field) => field,
                Err(error) => {
                    eprintln!("Error getting next field: {:?}", error);
                    return err();
                }
            } {
                let name = match field.name() {
                    Some(name) => name.to_string(),
                    None => {
                        eprintln!("Field without a name");
                        return err();
                    }
                };
                let data = match field.bytes().await {
                    Ok(data) => data,
                    Err(error) => {
                        eprintln!("Error reading bytes: {:?}", error);
                        return err();
                    }
                };

                if name == "new_dog_photo" {
                    // TODO: file_name should be the id of the newly created dog
                    // TODO: determine whether a file extension is required and how to steal the
                    // file extension from the old file
                    // let file_name = field.file_name().unwrap_or("unnamed_file").to_string();
                    let file_name = "blah2.jpg".to_string();
                    let file_path = Path::new("./unapproved").join(&file_name);

                    if let Err(error) = fs::write(&file_path, &data) {
                        eprintln!("Error saving file: {:?}", error);
                        return err();
                    }

                    println!("Saved file '{}' to {:?}", file_name, file_path);
                }
            }
            Html(upload_dog_form(FormField::empty()).into_string())
        }),
    )
}

pub fn upload_dog_form(new_dog_name: FormField<String>) -> Markup {
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
            input type="file" id="new_dog_photo" name="new_dog_photo" class="file-input file-input-bordered file-input-lg w-full" ;
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
