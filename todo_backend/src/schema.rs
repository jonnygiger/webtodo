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
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamp,
        expires_at -> Timestamp,
    }
}

diesel::joinable!(todo_items -> users (user_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    sessions,
    todo_items,
    users,
);
