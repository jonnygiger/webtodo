use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rocket::{get, post, put, routes, State};
use rocket::http::Status;
use rocket::request::FromParam;
use rocket::serde::json::Json;
use serde::{Deserialize, Deserializer, Serialize, Serializer}; // Removed 'de'
use std::ops::Deref;
use std::sync::RwLock;
use uuid::Uuid; // This is uuid v1.0
use std::hash::{Hash, Hasher};
use std::fmt; // Added for Display

// Newtype wrapper for uuid::Uuid
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUuid(pub Uuid);

impl AppUuid {
    pub fn new_v4() -> Self {
        AppUuid(Uuid::new_v4())
    }
}

// Implement Deref to allow AppUuid to behave like Uuid
impl Deref for AppUuid {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Manual FromParam implementation for AppUuid
impl<'r> FromParam<'r> for AppUuid {
    type Error = uuid::Error;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Uuid::parse_str(param).map(AppUuid)
    }
}

// Manual Serialize for AppUuid
impl Serialize for AppUuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

// Manual Deserialize for AppUuid
impl<'de> Deserialize<'de> for AppUuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Uuid::deserialize(deserializer).map(AppUuid)
    }
}

// Manual Hash for AppUuid
impl Hash for AppUuid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// Implement Display for AppUuid
impl fmt::Display for AppUuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f) // Delegate to inner Uuid's Display impl
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TodoItem {
    pub id: AppUuid, // Use AppUuid
    pub description: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Clone)]
pub struct TodoItemDescription {
    pub description: String,
}

pub type TodoStorage = RwLock<DashMap<AppUuid, TodoItem>>; // Use AppUuid as key

#[post("/todos", data = "<item_data>")]
pub async fn add_todo(storage: &State<TodoStorage>, item_data: Json<TodoItemDescription>) -> Json<TodoItem> {
    let id = AppUuid::new_v4(); // Create AppUuid
    let created_at = Utc::now();
    let new_item = TodoItem {
        id, // Store AppUuid
        description: item_data.description.clone(),
        completed: false,
        created_at,
    };

    let storage_map = storage.write().unwrap(); // Does not need to be mut for insert
    storage_map.insert(id, new_item.clone());

    Json(new_item)
}

#[get("/todos/<id>")]
pub async fn get_todo(storage: &State<TodoStorage>, id: AppUuid) -> Result<Json<TodoItem>, Status> { // id is AppUuid
    let storage_map = storage.read().unwrap();
    let result = if let Some(item_ref) = storage_map.get(&id) {
        let item_clone = item_ref.value().clone(); // Clone here
        Ok(Json(item_clone))
    } else {
        Err(Status::NotFound)
    };
    result
}

#[put("/todos/<id>/complete")]
pub async fn complete_todo(storage: &State<TodoStorage>, id: AppUuid) -> Result<Json<TodoItem>, Status> { // id is AppUuid
    let storage_map = storage.write().unwrap(); // Does not need to be mut for get_mut on DashMap via RwLockWriteGuard
    let result = if let Some(mut item_ref_mut) = storage_map.get_mut(&id) {
        item_ref_mut.completed = true;
        let item_clone = item_ref_mut.value().clone(); // Clone here
        Ok(Json(item_clone))
    } else {
        Err(Status::NotFound)
    };
    result
}

#[get("/todos/search?<description>")]
pub async fn search_todos(storage: &State<TodoStorage>, description: Option<String>) -> Json<Vec<TodoItem>> {
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
pub async fn list_todos_by_status(storage: &State<TodoStorage>, completed: Option<bool>) -> Json<Vec<TodoItem>> {
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
pub async fn get_todos_count(storage: &State<TodoStorage>) -> Json<usize> {
    let storage_map = storage.read().unwrap();
    Json(storage_map.len())
}

#[get("/todos/count?<completed>")]
pub async fn get_todos_count_by_status(storage: &State<TodoStorage>, completed: bool) -> Json<usize> {
    let storage_map = storage.read().unwrap();
    let count = storage_map
        .iter()
        .filter(|entry| entry.value().completed == completed)
        .count();
    Json(count)
}

// This function can be used by main.rs to launch the server
// and by tests to get a Rocket instance.
pub fn rocket_instance() -> rocket::Rocket<rocket::Build> {
    let todo_storage: TodoStorage = RwLock::new(DashMap::<AppUuid, TodoItem>::new()); // Use AppUuid
    rocket::build()
        .manage(todo_storage)
        .mount("/", routes![add_todo, get_todo, complete_todo, search_todos, list_todos_by_status, get_todos_count, get_todos_count_by_status])
}
