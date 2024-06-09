pub mod hello;
pub use hello::hello_router as hello;

pub mod count;
pub use count::count_router as count;

pub mod foo;
pub use foo::foo_router as foo;

pub mod contacts;
pub use contacts::contacts_router as contacts;

pub mod todo;
pub use todo::todo_router as todo;
