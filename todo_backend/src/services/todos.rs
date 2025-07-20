use crate::db::PgPool;
use crate::models::{NewTodoItem, TodoItem, CreateTodoRequest, TodoSearchQuery};
use crate::schema::todo_items;
use diesel::prelude::*;
use rocket::State;
use rocket::serde::json::Json;
use uuid::Uuid;
use crate::AuthenticatedUser;
use super::error::ServiceError;

pub fn add_todo_item(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    create_req: Json<CreateTodoRequest>,
) -> Result<Json<TodoItem>, ServiceError> {
    use todo_items::dsl::*;
    let mut conn = pool.get().map_err(|e| ServiceError::InternalError(format!("DB Connection error: {}", e)))?;

    let new_item = NewTodoItem {
        user_id: auth_user.user_id,
        description: create_req.description.clone(),
    };

    let item = diesel::insert_into(todo_items)
        .values(&new_item)
        .get_result::<TodoItem>(&mut conn)
        .map_err(|e| ServiceError::InternalError(format!("Failed to create todo item: {}", e)))?;
    Ok(Json(item))
}

pub fn get_todo_item(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    item_id_str: String,
) -> Result<Json<TodoItem>, ServiceError> {
    use todo_items::dsl::*;
    let mut conn = pool.get().map_err(|e| ServiceError::InternalError(format!("DB Connection error: {}", e)))?;
    let item_uuid = Uuid::parse_str(&item_id_str)
        .map_err(|_| ServiceError::InternalError("Invalid UUID format".to_string()))?;

    let item = todo_items
        .filter(id.eq(item_uuid).and(user_id.eq(auth_user.user_id)))
        .select(TodoItem::as_select())
        .first::<TodoItem>(&mut conn)
        .optional()
        .map_err(|e| ServiceError::InternalError(format!("DB query error: {}", e)))?;

    match item {
        Some(it) => Ok(Json(it)),
        None => Err(ServiceError::NotFound("Todo item not found".to_string())),
    }
}

pub fn complete_todo_item(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    item_id_str: String,
) -> Result<Json<TodoItem>, ServiceError> {
    use todo_items::dsl::*;
    let mut conn = pool.get().map_err(|e| ServiceError::InternalError(format!("DB Connection error: {}", e)))?;
    let item_uuid = Uuid::parse_str(&item_id_str)
        .map_err(|_| ServiceError::InternalError("Invalid UUID format".to_string()))?;

    let updated_item = diesel::update(todo_items.filter(id.eq(item_uuid).and(user_id.eq(auth_user.user_id))))
        .set(completed.eq(true))
        .get_result::<TodoItem>(&mut conn)
        .optional() // Use optional to handle not found case
        .map_err(|e| ServiceError::InternalError(format!("DB update error: {}", e)))?;

    match updated_item {
        Some(it) => Ok(Json(it)),
        None => Err(ServiceError::NotFound("Todo item not found or not owned by user".to_string())),
    }
}

pub fn list_or_search_todos(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    search_query: TodoSearchQuery,
) -> Result<Json<Vec<TodoItem>>, ServiceError> {
    use todo_items::dsl::*;
    let mut conn = pool.get().map_err(|e| ServiceError::InternalError(format!("DB Connection error: {}", e)))?;

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
        .map_err(|e| ServiceError::InternalError(format!("DB query error: {}", e)))?;

    Ok(Json(items))
}

pub fn get_todos_count(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    search_query: TodoSearchQuery,
) -> Result<Json<i64>, ServiceError> {
    use todo_items::dsl::*;
    let mut conn = pool.get().map_err(|e| ServiceError::InternalError(format!("DB Connection error: {}", e)))?;

    let mut query = todo_items
        .filter(user_id.eq(auth_user.user_id))
        .into_boxed();

    if let Some(ref desc_filter) = search_query.description {
         query = query.filter(description.ilike(format!("%{}%", desc_filter)));
    }
    if let Some(comp_filter) = search_query.completed {
        query = query.filter(completed.eq(comp_filter));
    }

    let count_val = query
        .count()
        .get_result(&mut conn)
        .map_err(|e| ServiceError::InternalError(format!("DB count query error: {}", e)))?;

    Ok(Json(count_val))
}
