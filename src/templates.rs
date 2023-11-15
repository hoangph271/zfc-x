use askama::Template;
use super::dto::Todo;

#[derive(Template)]
#[template(path = "index.html")]
pub struct HelloTemplate;

#[derive(Template)]
#[template(path = "stream.html")]
pub struct StreamTemplate;

#[derive(Template)]
#[template(path = "todos.html")]
pub struct Records {
    pub todos: Vec<Todo>,
}

#[derive(Template)]
#[template(path = "todo.html")]
pub struct TodoNewTemplate {
    pub todo: Todo,
}
