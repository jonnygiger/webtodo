extern crate rocket; // Removed #[macro_use]
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy; // Added for GLOBAL_USER_ID
use dashmap::DashMap;
use rocket::{get, post, put, routes, State}; // State is already here
use rocket::fs::{FileServer, relative}; // Removed NamedFile
use rocket::http::Status;
use rocket::request::FromParam;
use rocket::serde::json::Json;
use serde::{Deserialize, Deserializer, Serialize, Serializer}; // Removed 'de'
use std::ops::Deref;
use std::sync::RwLock;
use uuid::Uuid; // This is uuid v1.0
use std::hash::{Hash, Hasher};
use std::fmt; // Added for Display
use rocket::form::{self, FromFormField, ValueField}; // Required for UserId manual impl
use bcrypt::{hash, DEFAULT_COST, verify}; // Added verify for login
// Removed: use rocket::request::Request;
// State is already imported via line 4 use rocket::{..., State};
// Uuid is imported on line 12, needed for GLOBAL_USER_ID if parsing from string, or if UserId constructor needs it.
// UserId struct is defined below.
// Uuid is imported on line 12.

const GLOBAL_USER_UUID_STR: &str = "018f9db0-0c9f-7008-9089-47110058134A";
static GLOBAL_USER_ID: Lazy<UserId> = Lazy::new(|| UserId(Uuid::parse_str(GLOBAL_USER_UUID_STR).expect("Failed to parse GLOBAL_USER_UUID_STR")));

// Newtype wrapper for uuid::Uuid
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUuid(pub Uuid);

impl AppUuid {
    pub fn new_v4() -> Self {
        AppUuid(Uuid::new_v4())
    }
}

// It's good practice to define a newtype for User IDs as well for type safety
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)] // Removed FromFormField (manual impl)
#[serde(transparent)]
pub struct UserId(pub Uuid); // Made Uuid public for construction

impl UserId {
    pub fn new() -> Self {
        UserId(Uuid::new_v4())
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// Manual FromParam implementation for UserId (already existed, good)
impl<'r> FromParam<'r> for UserId {
    type Error = uuid::Error;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Uuid::parse_str(param).map(UserId)
    }
}

// Manual FromFormField implementation for UserId
#[rocket::async_trait]
impl<'r> FromFormField<'r> for UserId {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        match Uuid::parse_str(field.value) {
            Ok(uuid) => Ok(UserId(uuid)),
            Err(_) => Err(form::Error::validation("Invalid UUID string for UserId").into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub password_hash: String,
}

pub type UserStore = DashMap<UserId, User>;

#[derive(Deserialize)]
pub struct UserRegistration<'r> {
    username: &'r str,
    password: &'r str,
}

#[derive(Deserialize)]
pub struct UserLogin<'r> {
    username: &'r str,
    password: &'r str,
}

#[derive(Serialize, Deserialize, Debug)] // Added Deserialize and Debug
pub struct LoginResponse {
    pub session_token: String, // Made fields public
    pub user_id: UserId,       // Made fields public
    pub username: String,      // Made fields public
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
    pub user_id: UserId, // Added user_id field
    pub description: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Clone)]
pub struct TodoItemDescription {
    pub description: String,
}

// Renaming TodoStorage to TodoStore for consistency and defining TodoApp
pub type TodoStore = RwLock<DashMap<AppUuid, TodoItem>>; // Use AppUuid as key
pub type SessionStore = DashMap<String, UserId>; // SessionToken (String) -> UserId

pub struct TodoApp {
    pub todos: TodoStore,
    pub users: UserStore,
    pub sessions: SessionStore, // Added session store
}

// AuthenticatedUser guard is now fully removed.

impl TodoApp {
    pub fn new() -> Self {
        TodoApp {
            todos: RwLock::new(DashMap::new()),
            users: UserStore::new(),
            sessions: SessionStore::new(), // Initialize session store
        }
    }
}

#[post("/", data = "<item_data>")] // Changed from "/todos" to "/"
pub async fn add_todo(item_data: Json<TodoItemDescription>, app_state: &State<TodoApp>) -> Result<Json<TodoItem>, Status> {
    // Removed req: &Request<'_>
    // Using GLOBAL_USER_ID
    let user_id = *GLOBAL_USER_ID;

    let id = AppUuid::new_v4();
    let created_at = Utc::now();
    let new_item = TodoItem {
        id,
        user_id, // Use GLOBAL_USER_ID
        description: item_data.description.clone(),
        completed: false,
        created_at,
    };

    let storage_map = app_state.todos.write().unwrap();
    storage_map.insert(id, new_item.clone());

    Ok(Json(new_item))
}

#[get("/<id>")] // Changed from "/todos/<id>" to "/<id>"
pub async fn get_todo(id: AppUuid, app_state: &State<TodoApp>) -> Result<Json<TodoItem>, Status> {
    // Removed req: &Request<'_>
    // Using GLOBAL_USER_ID for check
    let current_user_id = *GLOBAL_USER_ID;

    let item_owned = {
        let storage_map = app_state.todos.read().unwrap();
        storage_map.get(&id).map(|item_ref| item_ref.value().clone())
    };

    if let Some(item) = item_owned {
        if item.user_id == current_user_id {
            Ok(Json(item))
        } else {
            Err(Status::NotFound)
        }
    } else {
        Err(Status::NotFound)
    }
}

#[put("/<id>/complete")] // Changed from "/todos/<id>/complete" to "/<id>/complete"
pub async fn complete_todo(id: AppUuid, app_state: &State<TodoApp>) -> Result<Json<TodoItem>, Status> {
    // Removed req: &Request<'_>
    // Using GLOBAL_USER_ID for check
    let current_user_id = *GLOBAL_USER_ID;

    let storage_map = app_state.todos.write().unwrap();
    let outcome = if let Some(mut item_ref_mut) = storage_map.get_mut(&id) {
        if item_ref_mut.user_id == current_user_id {
            item_ref_mut.completed = true;
            Ok(Json(item_ref_mut.value().clone()))
        } else {
            Err(Status::NotFound)
        }
    } else {
        Err(Status::NotFound)
    };
    outcome
}

#[get("/search?<description>")] // Changed from "/todos/search?<description>"
pub async fn search_todos(description: Option<String>, app_state: &State<TodoApp>) -> Result<Json<Vec<TodoItem>>, Status> {
    // Removed req: &Request<'_>
    // Using GLOBAL_USER_ID for filter
    let current_user_id = *GLOBAL_USER_ID;

    let storage_map = app_state.todos.read().unwrap();
    let items: Vec<TodoItem> = storage_map
        .iter()
        .filter(|entry| entry.value().user_id == current_user_id)
        .filter(|entry| match &description {
            Some(query) => entry.value().description.to_lowercase().contains(&query.to_lowercase()),
            None => true,
        })
        .map(|entry| entry.value().clone())
        .collect();
    Ok(Json(items))
}

#[get("/?<completed>")] // Corrected: Added leading /
pub async fn list_todos_by_status(completed: Option<bool>, app_state: &State<TodoApp>) -> Result<Json<Vec<TodoItem>>, Status> {
    // Removed req: &Request<'_>
    // Using GLOBAL_USER_ID for filter
    let current_user_id = *GLOBAL_USER_ID;

    let storage_map = app_state.todos.read().unwrap();
    let items: Vec<TodoItem> = storage_map
        .iter()
        .filter(|entry| entry.value().user_id == current_user_id)
        .filter(|entry| match completed {
            Some(status) => entry.value().completed == status,
            None => true,
        })
        .map(|entry| entry.value().clone())
        .collect();
    Ok(Json(items))
}

#[get("/count")] // Changed from "/todos/count"
pub async fn get_todos_count(app_state: &State<TodoApp>) -> Result<Json<usize>, Status> {
    // Removed req: &Request<'_>
    // Using GLOBAL_USER_ID for filter
    let current_user_id = *GLOBAL_USER_ID;

    let storage_map = app_state.todos.read().unwrap();
    let count = storage_map
        .iter()
        .filter(|entry| entry.value().user_id == current_user_id)
        .count();
    Ok(Json(count))
}

#[get("/count?<completed>")] // Changed from "/todos/count?<completed>"
pub async fn get_todos_count_by_status(completed: bool, app_state: &State<TodoApp>) -> Result<Json<usize>, Status> {
    // Removed req: &Request<'_>
    // Using GLOBAL_USER_ID for filter
    let current_user_id = *GLOBAL_USER_ID;

    let storage_map = app_state.todos.read().unwrap();
    let count = storage_map
        .iter()
        .filter(|entry| entry.value().user_id == current_user_id)
        .filter(|entry| entry.value().completed == completed)
        .count();
    Ok(Json(count))
}

#[get("/")]
async fn serve_index() -> Option<rocket::fs::NamedFile> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("static/index.html");
    rocket::fs::NamedFile::open(path).await.ok()
}

#[post("/register", data = "<user_data>")]
pub fn register(user_data: Json<UserRegistration<'_>>, app_state: &State<TodoApp>) -> Result<Json<User>, Status> {
    // Check if username already exists
    if app_state.users.iter().any(|entry| entry.value().username == user_data.username) {
        return Err(Status::Conflict); // Username already exists
    }

    // Hash the password
    let password_hash = match hash(user_data.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return Err(Status::InternalServerError), // Password hashing failed
    };

    let new_user_id = UserId::new();
    let new_user = User {
        id: new_user_id,
        username: user_data.username.to_string(),
        password_hash,
    };

    app_state.users.insert(new_user_id, new_user.clone());

    // Create a user object to return, excluding the password hash for security
    let user_response = User {
        id: new_user.id,
        username: new_user.username,
        password_hash: String::from(""), // Don't send hash back
    };

    Ok(Json(user_response))
}

#[post("/login", data = "<login_data>")]
pub fn login(login_data: Json<UserLogin<'_>>, app_state: &State<TodoApp>) -> Result<Json<LoginResponse>, Status> {
    let username = login_data.username;
    let password = login_data.password;

    // Find user by username
    let user_option: Option<User> = app_state.users.iter()
        .find(|entry| entry.value().username == username)
        .map(|entry| entry.value().clone()); // Clones the User struct itself

    match user_option {
        Some(user) => { // user is now of type User
            // Verify password
            match verify(password, &user.password_hash) {
                Ok(true) => {
                    // Password is correct, create a session token
                    let session_token = Uuid::new_v4().to_string();
                    app_state.sessions.insert(session_token.clone(), user.id);

                    let response = LoginResponse {
                        session_token,
                        user_id: user.id,
                        username: user.username.clone(),
                    };
                    Ok(Json(response))
                }
                Ok(false) => Err(Status::Unauthorized), // Invalid password
                Err(_) => Err(Status::InternalServerError), // bcrypt error
            }
        }
        None => Err(Status::NotFound), // User not found
    }
}

fn todo_routes() -> Vec<rocket::Route> {
    routes![add_todo, get_todo, complete_todo, search_todos, list_todos_by_status, get_todos_count, get_todos_count_by_status]
}

fn auth_routes() -> Vec<rocket::Route> {
    routes![register, login] // Added login route
}

// This function can be used by main.rs to launch the server
// and by tests to get a Rocket instance.
pub fn rocket_instance() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .manage(TodoApp::new()) // Manages both todos and users
        .mount("/", routes![serve_index]) // New route for index.html
        .mount("/static", FileServer::from(relative!("static"))) // New file server for static assets
        .mount("/api/todos", todo_routes()) // Grouped todo routes
        .mount("/auth", auth_routes()) // Added auth routes
}
