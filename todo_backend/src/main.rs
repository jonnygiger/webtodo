use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rocket::{get, post, put, State}; // Added put for later use
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TodoItem {
    id: Uuid,
    description: String,
    completed: bool,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Clone)]
struct TodoItemDescription {
    description: String,
}

type TodoStorage = RwLock<DashMap<Uuid, TodoItem>>;

#[post("/todos", data = "<item_data>")]
async fn add_todo(storage: &State<TodoStorage>, item_data: Json<TodoItemDescription>) -> Json<TodoItem> {
    let id = Uuid::new_v4();
    let created_at = Utc::now();
    let new_item = TodoItem {
        id,
        description: item_data.description.clone(),
        completed: false,
        created_at,
    };

    let mut storage_map = storage.write().unwrap();
    storage_map.insert(id, new_item.clone());

    Json(new_item)
}

#[get("/todos/<id>")]
async fn get_todo(storage: &State<TodoStorage>, id: Uuid) -> Result<Json<TodoItem>, Status> {
    let storage_map = storage.read().unwrap();
    match storage_map.get(&id) {
        Some(item) => Ok(Json(item.clone())),
        None => Err(Status::NotFound),
    }
}

#[rocket::main]
async fn main() {
    let todo_storage = RwLock::new(DashMap::new());
    rocket::build()
        .manage(todo_storage)
        .mount("/", routes![add_todo, get_todo, complete_todo, search_todos, list_todos_by_status, get_todos_count, get_todos_count_by_status])
        .launch()
        .await
        .expect("Rocket server failed to launch");
}

#[put("/todos/<id>/complete")]
async fn complete_todo(storage: &State<TodoStorage>, id: Uuid) -> Result<Json<TodoItem>, Status> {
    let mut storage_map = storage.write().unwrap();
    match storage_map.get_mut(&id) {
        Some(mut item) => {
            item.completed = true;
            Ok(Json(item.clone()))
        }
        None => Err(Status::NotFound),
    }
}

#[get("/todos/search?<description>")]
async fn search_todos(storage: &State<TodoStorage>, description: Option<String>) -> Json<Vec<TodoItem>> {
    let storage_map = storage.read().unwrap();
    let items: Vec<TodoItem> = match description {
        Some(query) => storage_map
            .iter()
            .filter(|entry| {
                entry.value().description.to_lowercase().contains(&query.to_lowercase())
            })
            .map(|entry| entry.value().clone())
            .collect(),
        None => storage_map.iter().map(|entry| entry.value().clone()).collect(),
    };
    Json(items)
}

#[get("/todos?<completed>")]
async fn list_todos_by_status(storage: &State<TodoStorage>, completed: Option<bool>) -> Json<Vec<TodoItem>> {
    let storage_map = storage.read().unwrap();
    let items: Vec<TodoItem> = match completed {
        Some(status) => storage_map
            .iter()
            .filter(|entry| entry.value().completed == status)
            .map(|entry| entry.value().clone())
            .collect(),
        None => storage_map.iter().map(|entry| entry.value().clone()).collect(),
    };
    Json(items)
}

#[get("/todos/count")]
async fn get_todos_count(storage: &State<TodoStorage>) -> Json<usize> {
    let storage_map = storage.read().unwrap();
    Json(storage_map.len())
}

#[get("/todos/count?<completed>")]
async fn get_todos_count_by_status(storage: &State<TodoStorage>, completed: bool) -> Json<usize> {
    let storage_map = storage.read().unwrap();
    let count = storage_map
        .iter()
        .filter(|entry| entry.value().completed == completed)
        .count();
    Json(count)
}
