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
    let mut conn = pool.get().map_err(|_| ServiceError::InternalError("Failed to get DB connection".to_string()))?;

    let new_item = NewTodoItem {
        user_id: auth_user.user_id,
        description: create_req.description.clone(),
    };

    let item = diesel::insert_into(todo_items)
        .values(&new_item)
        .get_result::<TodoItem>(&mut conn)?;
    Ok(Json(item))
}

pub fn get_todo_item(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    item_id_str: String,
) -> Result<Json<TodoItem>, ServiceError> {
    use todo_items::dsl::*;
    let mut conn = pool.get().map_err(|_| ServiceError::InternalError("Failed to get DB connection".to_string()))?;
    let item_uuid = Uuid::parse_str(&item_id_str)
        .map_err(|_| ServiceError::InvalidInput("Invalid UUID format".to_string()))?;

    let item = todo_items
        .filter(id.eq(item_uuid).and(user_id.eq(auth_user.user_id)))
        .select(TodoItem::as_select())
        .first::<TodoItem>(&mut conn)
        .optional()?;

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
    let mut conn = pool.get().map_err(|_| ServiceError::InternalError("Failed to get DB connection".to_string()))?;
    let item_uuid = Uuid::parse_str(&item_id_str)
        .map_err(|_| ServiceError::InvalidInput("Invalid UUID format".to_string()))?;

    let updated_item = diesel::update(todo_items.filter(id.eq(item_uuid).and(user_id.eq(auth_user.user_id))))
        .set(completed.eq(true))
        .get_result::<TodoItem>(&mut conn)
        .optional()?;

    match updated_item {
        Some(it) => Ok(Json(it)),
        None => Err(ServiceError::NotFound("Todo item not found or not owned by user".to_string())),
    }
}

fn _build_todo_query(
    auth_user: &AuthenticatedUser,
    search_query: &TodoSearchQuery,
) -> diesel::query_builder::BoxedSelectStatement<
    'static,
    (
        diesel::sql_types::Uuid,
        diesel::sql_types::Uuid,
        diesel::sql_types::Text,
        diesel::sql_types::Bool,
        diesel::sql_types::Timestamptz,
    ),
    todo_items::table,
    diesel::pg::Pg,
> {
    use todo_items::dsl::*;
    let mut query = todo_items
        .filter(user_id.eq(auth_user.user_id))
        .into_boxed();

    if let Some(ref desc_filter) = search_query.description {
        query = query.filter(description.ilike(format!("%{}%", desc_filter)));
    }
    if let Some(comp_filter) = search_query.completed {
        query = query.filter(completed.eq(comp_filter));
    }
    query
}

pub fn list_or_search_todos(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    search_query: TodoSearchQuery,
) -> Result<Json<Vec<TodoItem>>, ServiceError> {
    use todo_items::dsl::*;
    let mut conn = pool.get().map_err(|_| ServiceError::InternalError("Failed to get DB connection".to_string()))?;

    let query = _build_todo_query(&auth_user, &search_query);

    let items = query
        .order(created_at.desc())
        .select(TodoItem::as_select())
        .load::<TodoItem>(&mut conn)?;

    Ok(Json(items))
}

pub fn get_todos_count(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    search_query: TodoSearchQuery,
) -> Result<Json<i64>, ServiceError> {
    let mut conn = pool.get().map_err(|_| ServiceError::InternalError("Failed to get DB connection".to_string()))?;

    let query = _build_todo_query(&auth_user, &search_query);

    let count_val = query
        .count()
        .get_result(&mut conn)?;

    Ok(Json(count_val))
}

pub fn delete_todo_item(
    pool: &State<PgPool>,
    auth_user: AuthenticatedUser,
    item_id_str: String,
) -> Result<(), ServiceError> {
    use todo_items::dsl::*;
    let mut conn = pool.get().map_err(|_| ServiceError::InternalError("Failed to get DB connection".to_string()))?;
    let item_uuid = Uuid::parse_str(&item_id_str)
        .map_err(|_| ServiceError::InvalidInput("Invalid UUID format".to_string()))?;

    let target = todo_items.filter(id.eq(item_uuid).and(user_id.eq(auth_user.user_id)));

    let num_deleted = diesel::delete(target)
        .execute(&mut conn)?;

    if num_deleted > 0 {
        Ok(())
    } else {
        Err(ServiceError::NotFound("Todo item not found or not owned by user".to_string()))
    }
}
