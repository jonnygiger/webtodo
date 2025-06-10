// todo_backend/src/main.rs
use todo_backend::rocket_instance; // Use the lib's rocket_instance

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // Load .env file for database URL and other configurations
    dotenvy::dotenv().ok(); // Use dotenvy

    let _rocket = rocket_instance()
        .launch()
        .await?;
    Ok(())
}
