// todo_backend/src/db.rs
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use rocket::fairing::AdHoc;
use std::env;
use once_cell::sync::Lazy; // Add this if not already used

// an R2D2 connection pool
pub type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// DATABASE_URL static variable using once_cell
static DATABASE_URL: Lazy<String> = Lazy::new(|| {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
});

/// Initialize the database pool.
pub fn init_pool() -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.as_str());
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database pool")
}

// Fairing for attaching the pool to Rocket's managed state
pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel PostgreSQL Pool", |rocket| async {
        let pool = init_pool();
        // Run migrations on startup
        // Note: In a production app, you might want to handle this differently,
        // e.g., running migrations as a separate step during deployment.
        // For simplicity here, we run them on application start.
        // This requires `diesel_migrations` feature to be enabled for diesel CLI and accessible.
        // However, diesel_migrations crate itself isn't directly used here in code for running them.
        // The `embed_migrations!` macro and `run_pending_migrations` are part of `diesel_migrations` crate.
        // We will need to add `diesel_migrations` to `lib.rs` or `main.rs` to run them.
        // For now, we'll assume migrations are run manually or by a script before app start in production.
        // For testing and CI, we will manage this.
        rocket.manage(pool)
    })
}
