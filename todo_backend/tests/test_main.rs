#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use serde_json::json;
    use todo_backend::models::{TodoItem, UserInfo};
    use todo_backend::LoginResponse;
    use uuid::Uuid; // For Uuid parsing

    // Helper function to create a test client
    fn test_client() -> Client {
        let rocket_instance = todo_backend::rocket_instance();
        Client::tracked(rocket_instance).expect("valid rocket instance")
    }

    // --- New Authentication Tests ---
    #[test]
    fn test_register_user_success() {
        let client = test_client();
        // Use a unique username for this test to avoid conflicts with other tests
        let username = format!("testuser_reg_{}", Uuid::new_v4());
        let password = "password123";
        let response = client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let user_res = response.into_json::<UserInfo>().unwrap(); // User is from lib.rs
        assert_eq!(user_res.username, username);
    }

    #[test]
    fn test_register_user_conflict() {
        let client = test_client();
        let username = format!("testuser_conflict_{}", Uuid::new_v4());
        let password = "password123";
        // First registration
        client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        // Second registration with same username
        let response = client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(response.status(), Status::Conflict);
    }

    #[test]
    fn test_login_user_success() {
        let client = test_client();
        let username = format!("testuser_login_{}", Uuid::new_v4());
        let password = "password123";
        // Register user first
        let reg_resp = client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(reg_resp.status(), Status::Ok, "Registration failed: {:?}", reg_resp.into_string());

        // Attempt login
        let response = client.post("/auth/login")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(response.status(), Status::Ok, "Login failed: {:?}", response.into_string());

        // LoginResponse is from lib.rs and should be pub
        let login_res = response.into_json::<LoginResponse>().unwrap();
        assert!(!login_res.session_token.is_empty());
        assert_eq!(login_res.username, username);
    }

    #[test]
    fn test_login_user_not_found() {
        let client = test_client();
        let response = client.post("/auth/login")
            .header(ContentType::JSON)
            .body(json!({ "username": "nonexistentuser_test", "password": "password" }).to_string())
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_login_user_wrong_password() {
        let client = test_client();
        let username = format!("testuser_wrongpass_{}", Uuid::new_v4());
        let password = "password123";
        client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        let response = client.post("/auth/login")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": "wrongpassword" }).to_string())
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
    }

    // --- Updated Todo Tests ---
    #[test]
    fn test_add_todo() {
        let client = test_client();
        let username = format!("testuser_add_todo_{}", Uuid::new_v4());
        let password = "password123";

        // Register user
        let reg_response = client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(reg_response.status(), Status::Ok, "Registration failed");
        let user_info = reg_response.into_json::<UserInfo>().unwrap();
        let user_id = user_info.id; // Corrected: user_id

        // Login user
        let login_response = client.post("/auth/login")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(login_response.status(), Status::Ok, "Login failed");
        let login_info = login_response.into_json::<LoginResponse>().unwrap();
        let token = login_info.session_token;

        // Add todo item
        let description = "Test todo item - add";
        let response = client.post("/api/todos")
            .header(ContentType::JSON)
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .body(json!({ "description": description }).to_string())
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let item = response.into_json::<TodoItem>().unwrap();
        assert_eq!(item.description, description);
        assert!(!item.completed);
        assert_eq!(item.user_id, user_id, "UserId does not match the logged-in user's ID");
    }

    #[test]
    fn test_get_todo() {
        let client = test_client();
        let username = format!("testuser_get_todo_{}", Uuid::new_v4());
        let password = "password123";

        // Register user
        let reg_response = client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(reg_response.status(), Status::Ok, "Registration failed");
        let user_info = reg_response.into_json::<UserInfo>().unwrap();
        let user_id = user_info.id;

        // Login user
        let login_response = client.post("/auth/login")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(login_response.status(), Status::Ok, "Login failed");
        let login_info = login_response.into_json::<LoginResponse>().unwrap();
        let token = login_info.session_token;

        // Add todo item
        let description = "Test todo item - get";
        let add_response = client.post("/api/todos")
            .header(ContentType::JSON)
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .body(json!({ "description": description }).to_string())
            .dispatch();
        assert_eq!(add_response.status(), Status::Ok);
        let added_item = add_response.into_json::<TodoItem>().unwrap();
        let item_id = added_item.id;
        assert_eq!(added_item.user_id, user_id, "UserId on add does not match the logged-in user's ID");

        // Get todo item
        let response = client.get(format!("/api/todos/{}", item_id))
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let fetched_item = response.into_json::<TodoItem>().unwrap();
        assert_eq!(fetched_item.id, item_id);
        assert_eq!(fetched_item.description, description);
        assert_eq!(fetched_item.user_id, user_id, "UserId on get does not match the logged-in user's ID");

        // Test retrieving a non-existing item
        let non_existing_id = Uuid::new_v4();
        let response_not_found = client.get(format!("/api/todos/{}", non_existing_id))
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(response_not_found.status(), Status::NotFound);
    }

    #[test]
    fn test_complete_todo() {
        let client = test_client();
        let username = format!("testuser_complete_todo_{}", Uuid::new_v4());
        let password = "password123";

        // Register user
        let reg_response = client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(reg_response.status(), Status::Ok, "Registration failed");
        let user_info = reg_response.into_json::<UserInfo>().unwrap();
        let user_id = user_info.id;

        // Login user
        let login_response = client.post("/auth/login")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(login_response.status(), Status::Ok, "Login failed");
        let login_info = login_response.into_json::<LoginResponse>().unwrap();
        let token = login_info.session_token;

        // Add todo item
        let description = "Test todo item - complete";
        let add_response = client.post("/api/todos")
            .header(ContentType::JSON)
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .body(json!({ "description": description }).to_string())
            .dispatch();
        assert_eq!(add_response.status(), Status::Ok);
        let added_item = add_response.into_json::<TodoItem>().unwrap();
        let item_id = added_item.id;
        assert_eq!(added_item.user_id, user_id);

        // Complete todo item
        let complete_response = client.put(format!("/api/todos/{}/complete", item_id))
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(complete_response.status(), Status::Ok);
        let completed_item = complete_response.into_json::<TodoItem>().unwrap();
        assert_eq!(completed_item.id, item_id);
        assert!(completed_item.completed);
        assert_eq!(completed_item.user_id, user_id);

        // Get todo item to verify completion
        let get_response = client.get(format!("/api/todos/{}", item_id))
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(get_response.status(), Status::Ok);
        let fetched_item = get_response.into_json::<TodoItem>().unwrap();
        assert!(fetched_item.completed);
        assert_eq!(fetched_item.user_id, user_id);

        // Test completing a non-existing item
        let non_existing_id = Uuid::new_v4();
        let response_not_found = client.put(format!("/api/todos/{}/complete", non_existing_id))
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(response_not_found.status(), Status::NotFound);
    }

    #[test]
    fn test_search_todos() {
        let client = test_client();
        let username = format!("testuser_search_todos_{}", Uuid::new_v4());
        let password = "password123";

        // Register user
        let reg_response = client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(reg_response.status(), Status::Ok, "Registration failed");
        let user_info = reg_response.into_json::<UserInfo>().unwrap();
        let user_id = user_info.id;

        // Login user
        let login_response = client.post("/auth/login")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(login_response.status(), Status::Ok, "Login failed");
        let login_info = login_response.into_json::<LoginResponse>().unwrap();
        let token = login_info.session_token;

        // Add todo items
        client.post("/api/todos").header(ContentType::JSON).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).body(json!({ "description": "Learn Rust for this user" }).to_string()).dispatch();
        client.post("/api/todos").header(ContentType::JSON).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).body(json!({ "description": "Learn Rocket for this user" }).to_string()).dispatch();
        client.post("/api/todos").header(ContentType::JSON).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).body(json!({ "description": "Build an API for this user" }).to_string()).dispatch();

        // Search todo items
        let response = client.get("/api/todos?description=learn")
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let items = response.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items.len(), 2);
        for item in items {
            assert_eq!(item.user_id, user_id);
            assert!(item.description.to_lowercase().contains("learn"));
        }

        // Search all todo items for the user
        let response_all = client.get("/api/todos")
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(response_all.status(), Status::Ok);
        let items_all = response_all.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_all.len(), 3);
        for item in items_all {
            assert_eq!(item.user_id, user_id);
        }
    }

    #[test]
    fn test_list_todos_by_status() {
        let client = test_client();
        let username = format!("testuser_list_status_{}", Uuid::new_v4());
        let password = "password123";

        // Register user
        let reg_response = client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(reg_response.status(), Status::Ok, "Registration failed");
        let user_info = reg_response.into_json::<UserInfo>().unwrap();
        let user_id = user_info.id;

        // Login user
        let login_response = client.post("/auth/login")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(login_response.status(), Status::Ok, "Login failed");
        let login_info = login_response.into_json::<LoginResponse>().unwrap();
        let token = login_info.session_token;

        // Add todo items
        client.post("/api/todos").header(ContentType::JSON).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).body(json!({ "description": "Pending Task List" }).to_string()).dispatch();
        client.post("/api/todos").header(ContentType::JSON).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).body(json!({ "description": "Another Pending List" }).to_string()).dispatch();
        let r3 = client.post("/api/todos").header(ContentType::JSON).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).body(json!({ "description": "Completed Task List" }).to_string()).dispatch();
        let item3_id = r3.into_json::<TodoItem>().unwrap().id;
        client.put(format!("/api/todos/{}/complete", item3_id)).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).dispatch();

        // List completed todos
        let response_true = client.get("/api/todos?completed=true")
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(response_true.status(), Status::Ok);
        let items_true = response_true.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_true.len(), 1);
        for item in &items_true {
            assert!(item.completed);
            assert_eq!(item.user_id, user_id);
        }
        if !items_true.is_empty() {
             assert_eq!(items_true[0].description, "Completed Task List");
        }

        // List pending todos
        let response_false = client.get("/api/todos?completed=false")
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(response_false.status(), Status::Ok);
        let items_false = response_false.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_false.len(), 2);
        for item in items_false {
            assert!(!item.completed);
            assert_eq!(item.user_id, user_id);
        }

        // List all todos for the user
        let response_all = client.get("/api/todos")
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(response_all.status(), Status::Ok);
        let items_all = response_all.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_all.len(), 3);
        for item in items_all {
            assert_eq!(item.user_id, user_id);
        }
    }

    #[test]
    fn test_get_todos_count() {
        let client = test_client();
        let username = format!("testuser_count_todos_{}", Uuid::new_v4());
        let password = "password123";

        // Register user
        let reg_response = client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(reg_response.status(), Status::Ok, "Registration failed");
        let user_info = reg_response.into_json::<UserInfo>().unwrap();
        let _user_id = user_info.id; // Corrected: _user_id

        // Login user
        let login_response = client.post("/auth/login")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(login_response.status(), Status::Ok, "Login failed");
        let login_info = login_response.into_json::<LoginResponse>().unwrap();
        let token = login_info.session_token;

        // Add todo items
        client.post("/api/todos").header(ContentType::JSON).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).body(json!({ "description": "Count Item 1" }).to_string()).dispatch();
        client.post("/api/todos").header(ContentType::JSON).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).body(json!({ "description": "Count Item 2" }).to_string()).dispatch();

        // Get todos count
        let response_after_add = client.get("/api/todos/count")
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(response_after_add.status(), Status::Ok);
        assert_eq!(response_after_add.into_json::<usize>().unwrap(), 2); // Expect 2 items for this user
    }

    #[test]
    fn test_get_todos_count_by_status() {
        let client = test_client();
        let username = format!("testuser_count_status_{}", Uuid::new_v4());
        let password = "password123";

        // Register user
        let reg_response = client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(reg_response.status(), Status::Ok, "Registration failed");
        let user_info = reg_response.into_json::<UserInfo>().unwrap();
        let _user_id = user_info.id; // Corrected: _user_id

        // Login user
        let login_response = client.post("/auth/login")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(login_response.status(), Status::Ok, "Login failed");
        let login_info = login_response.into_json::<LoginResponse>().unwrap();
        let token = login_info.session_token;

        // Add todo items
        client.post("/api/todos").header(ContentType::JSON).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).body(json!({ "description": "Count Pending Status 1" }).to_string()).dispatch();
        client.post("/api/todos").header(ContentType::JSON).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).body(json!({ "description": "Count Pending Status 2" }).to_string()).dispatch();
        let r3 = client.post("/api/todos").header(ContentType::JSON).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).body(json!({ "description": "Count Completed Status 1" }).to_string()).dispatch();
        let item3_id = r3.into_json::<TodoItem>().unwrap().id;
        client.put(format!("/api/todos/{}/complete", item3_id)).header(rocket::http::Header::new("Authorization", format!("Bearer {}", token))).dispatch();

        // Get count of completed todos
        let response_true = client.get("/api/todos/count?completed=true")
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(response_true.status(), Status::Ok);
        assert_eq!(response_true.into_json::<usize>().unwrap(), 1, "Completed count mismatch");

        // Get count of pending todos
        let response_false = client.get("/api/todos/count?completed=false")
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(response_false.status(), Status::Ok);
        assert_eq!(response_false.into_json::<usize>().unwrap(), 2, "Pending count mismatch");
    }

    // --- Logout Test ---

    #[test]
    fn test_logout_and_attempt_access() {
        let client = test_client();
        let username = format!("testuser_logout_{}", Uuid::new_v4());
        let password = "password123";

        // Register user
        let reg_response = client.post("/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(reg_response.status(), Status::Ok, "Registration failed");

        // Login user
        let login_response = client.post("/auth/login")
            .header(ContentType::JSON)
            .body(json!({ "username": username, "password": password }).to_string())
            .dispatch();
        assert_eq!(login_response.status(), Status::Ok, "Login failed");
        let login_info = login_response.into_json::<LoginResponse>().unwrap();
        let token = login_info.session_token;

        // Logout
        let logout_response = client.post("/auth/logout")
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(logout_response.status(), Status::NoContent, "Logout request failed");

        // Attempt to use the token again
        let subsequent_access_response = client.get("/api/todos")
            .header(rocket::http::Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(subsequent_access_response.status(), Status::Unauthorized, "Access after logout did not fail as expected");

        // Verify error message for invalid token (as it's removed from session store)
        let body = subsequent_access_response.into_string().unwrap();
        assert!(body.contains("invalid_token"), "Error message for invalid token not found after logout. Body: {}", body);
    }
}
