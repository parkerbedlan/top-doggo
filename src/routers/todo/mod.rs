mod create_task;
mod delete_task;
mod read_tasks;
mod update_task;

use self::{
    create_task::create_task, delete_task::delete_task, read_tasks::todo_home,
    update_task::update_task,
};
use crate::AppState;
use axum::{
    routing::{delete, get},
    Router,
};
use maud::{html, Markup, Render};

pub fn todo_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/", get(todo_home).post(create_task).patch(update_task))
        .route("/:id", delete(delete_task))
}

struct Task {
    id: i64,
    description: String,
    done: bool,
}
impl Render for Task {
    fn render(&self) -> Markup {
        html! {
            div class="flex gap-2 w-full items-center"
            {
                button
                    class="text-error"
                    hx-delete={"/todo/" (self.id)}
                    hx-target="closest div"
                    hx-swap="outerHTML"
                    hx-confirm={"Are you sure you want to delete " (self.description) "?"}
                    {"X"}
                input
                    id={"checkbox-task-" (self.id)}
                    type="checkbox"
                    checked[self.done]
                    hx-trigger="change"
                    hx-patch="/todo"
                    hx-vals="js:{checked: event.target.checked, id: Number(event.target.id.split('-')[2])}"
                    hx-swap="none"
                    ;
                label
                    for={"checkbox-task-" (self.id)}
                    {(self.description)}
            }
        }
    }
}
