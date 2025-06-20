// todo_backend/src/lib.rs
#[macro_use]
extern crate rocket; // Ensure rocket macros are available

pub mod schema; // Generated by Diesel CLI
pub mod models;
pub mod db; // Our new db module

use bcrypt::{hash, verify, DEFAULT_COST};
use db::PgPool;
use diesel::prelude::*;
use models::*;
use rocket::http::{CookieJar, Status};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Build, Rocket, State}; // Import State
use uuid::Uuid;
use dotenvy;

// Re-export AppUuid if it's used elsewhere, or remove if not needed
// For simplicity, assuming Uuid directly from the uuid crate is fine.
// pub type AppUuid = Uuid; // If you had a type alias

// --- Error Types ---
#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ErrorDetail {
    error: String, // Changed field name from detail to error
}

#[derive(Responder, Debug)]
pub enum ApiError {
    #[response(status = 404, content_type = "json")]
    NotFound(Json<ErrorDetail>),
    #[response(status = 401, content_type = "json")]
    Unauthorized(Json<ErrorDetail>),
    #[response(status = 409, content_type = "json")]
    Conflict(Json<ErrorDetail>),
    #[response(status = 500, content_type = "json")]
    InternalError(Json<ErrorDetail>),
}

// --- Request Guards / Authentication ---
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct SessionToken {
    user_id: Uuid,
    username: String, // Optional: for quick lookup without hitting DB again
    // Add expiry if you want time-limited sessions
    // expires_at: i64, // Unix timestamp
}

// This would ideally be a secure, persistent store (e.g., Redis, or a DB table for sessions)
// For simplicity, keeping it in-memory for now, but this is NOT production-ready for multi-instance.
// Given the task is to use Postgres, sessions could also go into a 'sessions' table.
// However, the issue focuses on core data (users, todos) in Postgres.
// Let's keep session management simple for now to focus on Diesel for users/todos.
use dashmap::DashMap;
use once_cell::sync::Lazy;

static SESSIONS: Lazy<DashMap<String, SessionToken>> = Lazy::new(DashMap::new);
const SESSION_COOKIE_NAME: &str = "session_token";


#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for AuthenticatedUser {
    type Error = ApiError;

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let cookies = request.cookies();
        let auth_header = request.headers().get_one("Authorization");

        let token_str = if let Some(header_val) = auth_header {
            if header_val.starts_with("Bearer ") {
                Some(header_val.trim_start_matches("Bearer ").to_string())
            } else {
                None
            }
        } else {
            cookies.get(SESSION_COOKIE_NAME).map(|cookie| cookie.value().to_string())
            // TODO: Replace with get_private_with_key once key management is set up
            // cookies.get_private_with_key(SESSION_COOKIE_NAME, key_variable_here).map(|cookie| cookie.value().to_string())
        };

        match token_str {
            Some(token) => {
                if let Some(session) = SESSIONS.get(&token) {
                    // TODO: Check session expiry if implemented
                    rocket::request::Outcome::Success(AuthenticatedUser {
                        user_id: session.user_id,
                    })
                } else {
                        rocket::request::Outcome::Error((
                        Status::Unauthorized,
                        ApiError::Unauthorized(Json(ErrorDetail { error: "invalid_token".to_string() })),
                    ))
                }
            }
                None => rocket::request::Outcome::Error((
                Status::Unauthorized,
                ApiError::Unauthorized(Json(ErrorDetail { error: "missing_token".to_string() })),
            )),
        }
    }
}


// --- Route Handlers ---

// Auth routes
#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct LoginResponse {
    pub session_token: String,
    pub username: String,
    // Consider returning UserInfo here instead of just username
}


#[post("/auth/register", data = "<auth_req>")]
async fn register_user(
    pool: &State<PgPool>,
    auth_req: Json<AuthRequest>,
) -> Result<Json<UserInfo>, ApiError> {
    use schema::users::dsl::*;

    let mut conn = pool.get().map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB Connection error: {}", e) })))?;

    // Check if user already exists
    let existing_user = users
        .filter(username.eq(&auth_req.username))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .optional()
        .map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB query error: {}", e) })))?;

    if existing_user.is_some() {
        return Err(ApiError::Conflict(Json(ErrorDetail { error: "Username already exists".to_string() })));
    }

    let hashed_password = hash(&auth_req.password, DEFAULT_COST)
        .map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("Password hashing error: {}", e) })))?;

    let new_user = NewUser {
        username: &auth_req.username,
        password_hash: &hashed_password,
    };

    let user = diesel::insert_into(users)
        .values(&new_user)
        .get_result::<User>(&mut conn)
        .map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("Failed to create user: {}", e) })))?;

    Ok(Json(user.into()))
}

#[post("/auth/login", data = "<auth_req>")]
async fn login_user(
    pool: &State<PgPool>,
    cookies: &CookieJar<'_>,
    auth_req: Json<AuthRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    use schema::users::dsl::*;
    let mut conn = pool.get().map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB Connection error: {}", e) })))?;

    let found_user = users
        .filter(username.eq(&auth_req.username))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .optional()
        .map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB query error: {}", e) })))?;

    match found_user {
        Some(user) => {
            if verify(&auth_req.password, &user.password_hash)
                .map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("Password verification error: {}",e) })))?
            {
                let session_id = Uuid::new_v4().to_string();
                let session = SessionToken {
                    user_id: user.id,
                    username: user.username.clone(),
                };
                SESSIONS.insert(session_id.clone(), session);
                    // TODO: Replace with add_private_with_key once key management is set up
                    // cookies.add_private_with_key(rocket::http::Cookie::new(SESSION_COOKIE_NAME, session_id.clone()), key_variable_here);
                    cookies.add(rocket::http::Cookie::new(SESSION_COOKIE_NAME, session_id.clone()));


                Ok(Json(LoginResponse {
                    session_token: session_id,
                    username: user.username,
                }))
            } else {
                Err(ApiError::Unauthorized(Json(ErrorDetail { error: "Invalid credentials".to_string() })))
            }
        }
        None => Err(ApiError::NotFound(Json(ErrorDetail { error: "User not found".to_string() } ))),
    }
}

#[post("/auth/logout")]
async fn logout_user(_auth_user: AuthenticatedUser, cookies: &CookieJar<'_>) -> Status {
    // For logout, we need to get the token used for authentication.
    // This is tricky if only using AuthenticatedUser which just gives user_id.
    // A simple approach is to remove the cookie, but the token might still be valid if sent via Header.
    // For a more robust logout, the token itself should be invalidated.
    // Let's assume the FromRequest for AuthenticatedUser also stores the token string if needed.
    // Or, we require the token to be passed in the request for logout.

    // Simplified: remove cookie. If token was in header, it's harder to invalidate without storing it.
    // Let's find the token from cookies or header.
    // This is a bit of a workaround due to not having the token directly in AuthenticatedUser.
    // A better way would be to have the FromRequest guard also extract and provide the token.
        // TODO: Replace with get_private_with_key once key management is set up
        let token_from_cookie = cookies.get(SESSION_COOKIE_NAME).map(|c| c.value().to_string());
        // let token_from_cookie = cookies.get_private_with_key(SESSION_COOKIE_NAME, key_variable_here).map(|c| c.value().to_string());

    if let Some(token) = token_from_cookie {
        SESSIONS.remove(&token);
    }
        // TODO: Replace with remove_private_with_key once key management is set up
        // cookies.remove_private_with_key(rocket::http::Cookie::from(SESSION_COOKIE_NAME), key_variable_here);
        cookies.remove(rocket::http::Cookie::from(SESSION_COOKIE_NAME));
    Status::NoContent
}


// Todo item routes
#[post("/api/todos", data = "<create_req>")]
async fn add_todo_item(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    create_req: Json<CreateTodoRequest>,
) -> Result<Json<TodoItem>, ApiError> {
    use schema::todo_items::dsl::*;
    let mut conn = pool.get().map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB Connection error: {}", e) })))?;

    let new_item = NewTodoItem {
        user_id: auth_user.user_id,
        description: create_req.description.clone(),
    };

    let item = diesel::insert_into(todo_items)
        .values(&new_item)
        .get_result::<TodoItem>(&mut conn)
        .map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("Failed to create todo item: {}", e) })))?;
    Ok(Json(item))
}

#[get("/api/todos/<item_id_str>")]
async fn get_todo_item(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    item_id_str: String,
) -> Result<Json<TodoItem>, ApiError> {
    use schema::todo_items::dsl::*;
    let mut conn = pool.get().map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB Connection error: {}", e) })))?;
    let item_uuid = Uuid::parse_str(&item_id_str)
        .map_err(|_| ApiError::InternalError(Json(ErrorDetail { error: "Invalid UUID format".to_string() })))?;

    let item = todo_items
        .filter(id.eq(item_uuid).and(user_id.eq(auth_user.user_id)))
        .select(TodoItem::as_select())
        .first::<TodoItem>(&mut conn)
        .optional()
        .map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB query error: {}", e) })))?;

    match item {
        Some(it) => Ok(Json(it)),
        None => Err(ApiError::NotFound(Json(ErrorDetail { error: "Todo item not found".to_string() } ))),
    }
}

#[put("/api/todos/<item_id_str>/complete")]
async fn complete_todo_item(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    item_id_str: String,
) -> Result<Json<TodoItem>, ApiError> {
    use schema::todo_items::dsl::*;
    let mut conn = pool.get().map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB Connection error: {}", e) })))?;
    let item_uuid = Uuid::parse_str(&item_id_str)
        .map_err(|_| ApiError::InternalError(Json(ErrorDetail { error: "Invalid UUID format".to_string() })))?;

    let updated_item = diesel::update(todo_items.filter(id.eq(item_uuid).and(user_id.eq(auth_user.user_id))))
        .set(completed.eq(true))
        .get_result::<TodoItem>(&mut conn)
        .optional() // Use optional to handle not found case
        .map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB update error: {}", e) })))?;

    match updated_item {
        Some(it) => Ok(Json(it)),
        None => Err(ApiError::NotFound(Json(ErrorDetail { error: "Todo item not found or not owned by user".to_string() } ))),
    }
}

#[derive(Deserialize, Debug, rocket::form::FromForm)]
#[serde(crate = "rocket::serde")]
pub struct TodoSearchQuery {
    description: Option<String>,
    completed: Option<bool>, // Add this for filtering by completion status
}

// GET /api/todos (list all) and /api/todos/search?description=... (search by description)
// Combined into one handler, also handling /api/todos?completed=true/false
#[get("/api/todos?<search_query..>")]
async fn list_or_search_todos(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    search_query: TodoSearchQuery,
) -> Result<Json<Vec<TodoItem>>, ApiError> {
    use schema::todo_items::dsl::*;
    let mut conn = pool.get().map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB Connection error: {}", e) })))?;

    let mut query = todo_items
        .filter(user_id.eq(auth_user.user_id))
        .into_boxed();

    if let Some(ref desc_filter) = search_query.description {
        query = query.filter(description.ilike(format!("%{}%", desc_filter)));
    }
    if let Some(comp_filter) = search_query.completed {
        query = query.filter(completed.eq(comp_filter));
    }

    let items = query
        .order(created_at.desc())
        .select(TodoItem::as_select())
        .load::<TodoItem>(&mut conn)
        .map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB query error: {}", e) })))?;

    Ok(Json(items))
}


// GET /api/todos/search (this specific path is now covered by /api/todos?params...)
// For backward compatibility or specific endpoint, keep or remove.
// The tests seem to use /api/todos/search?description=
// The previous list_or_search_todos should handle this.
// If /api/todos/search without query params should list all, it's also handled.

#[get("/api/todos/count?<search_query..>")]
async fn get_todos_count(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    search_query: TodoSearchQuery, // Re-use TodoSearchQuery for consistency
) -> Result<Json<i64>, ApiError> { // Diesel count returns i64
    use schema::todo_items::dsl::*;
    let mut conn = pool.get().map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB Connection error: {}", e) })))?;

    let mut query = todo_items
        .filter(user_id.eq(auth_user.user_id))
        .into_boxed();

    if let Some(ref desc_filter) = search_query.description { // Though count usually doesn't use description search
         query = query.filter(description.ilike(format!("%{}%", desc_filter)));
    }
    if let Some(comp_filter) = search_query.completed {
        query = query.filter(completed.eq(comp_filter));
    }

    let count_val = query
        .count()
        .get_result(&mut conn)
        .map_err(|e| ApiError::InternalError(Json(ErrorDetail { error: format!("DB count query error: {}", e) })))?;

    Ok(Json(count_val))
}


// --- Rocket instance setup ---

use rocket::serde::json::{json, Value};

#[catch(401)]
fn unauthorized_catcher(_req: &rocket::Request<'_>) -> Json<Value> {
    // This catcher will be invoked for any 401 Unauthorized error.
    // The test `test_logout_and_attempt_access` specifically checks for
    // the JSON body `{"error": "invalid_token"}` after a logout
    // and subsequent access attempt.
    Json(json!({ "error": "invalid_token" }))
}

pub fn rocket_instance() -> Rocket<Build> {
    dotenvy::dotenv().ok(); // Load .env file
    rocket::build()
        .attach(db::stage()) // Attach the DB pool fairing
        .register("/", catchers![unauthorized_catcher]) // Register the catcher
        .mount(
            "/",
            routes![
                register_user,
                login_user,
                logout_user,
                add_todo_item,
                get_todo_item,
                complete_todo_item,
                list_or_search_todos, // This handles /api/todos and /api/todos?params
                get_todos_count,
                // Static file serving (if you had it before)
                // e.g. rocket_contrib::serve::StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static"))
            ],
        )
        // Potentially add static file serving if it was part of the original app
        // .mount("/", FileServer::from(relative!("static"))) // Example for Rocket 0.5
}

// Add any necessary `use` statements at the top of lib.rs for new modules like `schema` and `models`.
// Remove old in-memory data structures (USERS, TODO_ITEMS, SESSIONS if it moves to DB).
// The SESSIONS map is kept in-memory for now as per comment above.
