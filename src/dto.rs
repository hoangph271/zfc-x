use serde::{Serialize, Deserialize};

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Todo {
    pub id: i32,
    pub description: String,
}
