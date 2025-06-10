// @generated automatically by Diesel CLI.

diesel::table! {
    todo_items (id) {
        id -> Uuid,
        user_id -> Uuid,
        description -> Text,
        completed -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(todo_items -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    todo_items,
    users,
);
