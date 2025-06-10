// todo_backend/src/models.rs
use crate::schema::{users, todo_items};
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Queryable, Identifiable, Selectable, Serialize, Debug, PartialEq, Clone)]
#[diesel(table_name = users)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)] // Password hash should not be sent to client
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
}

// For returning user info without password hash
#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        UserInfo {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}


#[derive(Queryable, Identifiable, Selectable, Associations, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[diesel(belongs_to(User))]
#[diesel(table_name = todo_items)]
#[serde(crate = "rocket::serde")]
pub struct TodoItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub description: String,
    pub completed: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = todo_items)]
#[serde(crate = "rocket::serde")]
pub struct NewTodoItem {
    pub user_id: Uuid,
    pub description: String,
}

// Used for creating a todo item from a request (user_id will be from auth)
#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CreateTodoRequest {
    pub description: String,
}
