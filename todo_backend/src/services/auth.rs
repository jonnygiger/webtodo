use crate::db::PgPool;
use crate::models::{NewUser, User, UserInfo};
use crate::schema::sessions;
use diesel::prelude::*;
use rocket::State;
use rocket::serde::json::Json;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::{AuthRequest, LoginResponse};
use chrono::{Utc, Duration};
use super::error::ServiceError;

pub fn register_user(
    pool: &State<PgPool>,
    auth_req: Json<AuthRequest>,
) -> Result<Json<UserInfo>, ServiceError> {
    use crate::schema::users::dsl::*;

    let mut conn = pool.get().map_err(|_| ServiceError::InternalError("Failed to get DB connection".to_string()))?;

    // Check if user already exists
    let existing_user = users
        .filter(username.eq(&auth_req.username))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .optional()?;

    if existing_user.is_some() {
        return Err(ServiceError::Conflict("Username already exists".to_string()));
    }

    let hashed_password = hash(&auth_req.password, DEFAULT_COST)?;

    let new_user = NewUser {
        username: &auth_req.username,
        password_hash: &hashed_password,
    };

    let user = diesel::insert_into(users)
        .values(&new_user)
        .get_result::<User>(&mut conn)?;

    Ok(Json(user.into()))
}

pub fn login_user(
    pool: &State<PgPool>,
    cookies: &rocket::http::CookieJar<'_>,
    auth_req: Json<AuthRequest>,
) -> Result<Json<LoginResponse>, ServiceError> {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().map_err(|_| ServiceError::InternalError("Failed to get DB connection".to_string()))?;

    let found_user = users
        .filter(username.eq(&auth_req.username))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .optional()?;

    match found_user {
        Some(user) => {
            if verify(&auth_req.password, &user.password_hash)?
            {
                let new_session = NewSession {
                    user_id: user.id,
                    expires_at: Utc::now().naive_utc() + Duration::days(1),
                };

                let session = diesel::insert_into(sessions::table)
                    .values(&new_session)
                    .get_result::<Session>(&mut conn)?;

                cookies.add(rocket::http::Cookie::new("session_token", session.id.to_string()));

                Ok(Json(LoginResponse {
                    session_token: session.id.to_string(),
                    username: user.username,
                }))
            } else {
                Err(ServiceError::Unauthorized("Invalid credentials".to_string()))
            }
        }
        None => Err(ServiceError::NotFound("User not found".to_string())),
    }
}

pub fn logout_user(
    pool: &State<PgPool>,
    cookies: &rocket::http::CookieJar<'_>,
) -> Result<(), ServiceError> {
    let session_token = match cookies.get("session_token") {
        Some(cookie) => cookie.value().to_string(),
        None => return Ok(()),
    };

    let session_uuid = Uuid::parse_str(&session_token)
        .map_err(|_| ServiceError::InvalidInput("Invalid session token".to_string()))?;

    let mut conn = pool.get().map_err(|_| ServiceError::InternalError("Failed to get DB connection".to_string()))?;

    diesel::delete(sessions::table.filter(sessions::id.eq(session_uuid)))
        .execute(&mut conn)?;

    cookies.remove(rocket::http::Cookie::from("session_token"));

    Ok(())
}

#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = sessions)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub expires_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSession {
    pub user_id: Uuid,
    pub expires_at: chrono::NaiveDateTime,
}
