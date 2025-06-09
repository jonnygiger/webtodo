// Allow main to be dead code when compiling for library tests
#[allow(dead_code)]
#[rocket::main]
async fn main() {
    // Use the instance from the library
    todo_backend::rocket_instance()
        .launch()
        .await
        .expect("Rocket server failed to launch");
}
