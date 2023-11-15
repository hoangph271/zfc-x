use axum::{
    extract::{Path, State},
    http::{StatusCode, Response},
    response::{sse::Event, IntoResponse, Sse},
    routing::{delete, get},
    Extension, Form, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::broadcast::{channel, Sender};
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt as _};

mod dto;
mod templates;

use dto::Todo;
use templates::*;

#[derive(Clone, Serialize, Debug)]
enum MutationKind {
    Create,
    Delete,
}

#[derive(Clone, Serialize, Debug)]
pub struct TodoUpdate {
    mutation_kind: MutationKind,
    id: i32,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
struct TodoNew {
    description: String,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

type TodosStream = Sender<TodoUpdate>;

async fn home() -> impl IntoResponse {
    HelloTemplate
}

async fn stream() -> impl IntoResponse {
    StreamTemplate
}

async fn create_todo(
    State(state): State<AppState>,
    Extension(tx): Extension<TodosStream>,
    Form(form): Form<TodoNew>,
) -> impl IntoResponse {
    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO TODOS (description) VALUES ($1) RETURNING id, description",
    )
    .bind(form.description)
    .fetch_one(&state.db)
    .await
    .unwrap();

    if let Err(e) = tx.send(TodoUpdate {
        mutation_kind: MutationKind::Create,
        id: todo.id,
    }) {
        eprintln!(
            "Tried to send log of record with ID {} created but something went wrong: {e}",
            todo.id
        );
    }

    TodoNewTemplate { todo }
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(tx): Extension<TodosStream>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM TODOS WHERE ID = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .unwrap();

    if let Err(e) = tx.send(TodoUpdate {
        mutation_kind: MutationKind::Delete,
        id,
    }) {
        eprintln!("Tried to send log of record with ID {id} created but something went wrong: {e}");
    }

    StatusCode::OK
}

async fn handle_stream(
    Extension(tx): Extension<TodosStream>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = tx.subscribe();
    let stream = BroadcastStream::new(rx);

    Sse::new(
        stream
            .map(|msg| {
                let msg = msg.unwrap();
                // wrap the message in HTML because htmx expects a HTML fragment response
                let json = format!("<div>{}</div>", json!(msg));
                Event::default().data(json)
            })
            .map(Ok),
    )
    .keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(600))
            .text("keep-alive-text"),
    )
}

async fn fetch_todos(State(state): State<AppState>) -> impl IntoResponse {
    let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
        .fetch_all(&state.db)
        .await
        .unwrap();

    Records { todos }
}

pub async fn styles() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(include_str!("../templates/styles.css").to_owned())
        .unwrap()
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] db: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Looks like something went wrong with migrations...!");

    let (tx, _rx) = channel::<TodoUpdate>(10);
    let state = AppState { db };

    let router = Router::new()
        .route("/", get(home))
        .route("/stream", get(stream))
        .route("/styles.css", get(styles))
        .route("/todos", get(fetch_todos).post(create_todo))
        .route("/todos/:id", delete(delete_todo))
        .route("/todos/stream", get(handle_stream))
        .with_state(state)
        .layer(Extension(tx));

    Ok(router.into())
}
