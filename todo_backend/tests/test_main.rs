#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use serde_json::json;
    use todo_backend::*; // Import all public items from todo_backend crate
    use uuid::Uuid; // For Uuid parsing

    // Define the same constant UUID string here for test comparisons
    const EXPECTED_GLOBAL_USER_UUID_STR: &str = "018f9db0-0c9f-7008-9089-47110058134A";

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
        let user_res = response.into_json::<User>().unwrap(); // User is from lib.rs
        assert_eq!(user_res.username, username);
        assert_eq!(user_res.password_hash, "");
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
        let description = "Test todo item - add";
        let response = client.post("/api/todos")
            .header(ContentType::JSON)
            .body(json!({ "description": description }).to_string())
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let item = response.into_json::<TodoItem>().unwrap();
        assert_eq!(item.description, description);
        assert!(!item.completed);

        let expected_user_uuid = Uuid::parse_str(EXPECTED_GLOBAL_USER_UUID_STR).unwrap();
        assert_eq!(item.user_id.0, expected_user_uuid, "UserId does not match GLOBAL_USER_ID");
    }

    #[test]
    fn test_get_todo() {
        let client = test_client();
        let description = "Test todo item - get";
        let expected_user_uuid = Uuid::parse_str(EXPECTED_GLOBAL_USER_UUID_STR).unwrap();

        let add_response = client.post("/api/todos")
            .header(ContentType::JSON)
            .body(json!({ "description": description }).to_string())
            .dispatch();
        assert_eq!(add_response.status(), Status::Ok);
        let added_item = add_response.into_json::<TodoItem>().unwrap();
        let item_id = added_item.id;
        assert_eq!(added_item.user_id.0, expected_user_uuid, "UserId on add does not match GLOBAL_USER_ID");

        let response = client.get(format!("/api/todos/{}", item_id)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let fetched_item = response.into_json::<TodoItem>().unwrap();
        assert_eq!(fetched_item.id, item_id);
        assert_eq!(fetched_item.description, description);
        assert_eq!(fetched_item.user_id.0, expected_user_uuid, "UserId on get does not match GLOBAL_USER_ID");

        // Test retrieving a non-existing item (this part remains the same, no user_id check needed)
        let non_existing_id = AppUuid::new_v4();
        let response_not_found = client.get(format!("/api/todos/{}", non_existing_id)).dispatch();
        assert_eq!(response_not_found.status(), Status::NotFound);
    }

    #[test]
    fn test_complete_todo() {
        let client = test_client();
        let description = "Test todo item - complete";
        let expected_user_uuid = Uuid::parse_str(EXPECTED_GLOBAL_USER_UUID_STR).unwrap();

        let add_response = client.post("/api/todos")
            .header(ContentType::JSON)
            .body(json!({ "description": description }).to_string())
            .dispatch();
        assert_eq!(add_response.status(), Status::Ok);
        let added_item = add_response.into_json::<TodoItem>().unwrap();
        let item_id = added_item.id;
        assert_eq!(added_item.user_id.0, expected_user_uuid);

        let complete_response = client.put(format!("/api/todos/{}/complete", item_id)).dispatch();
        assert_eq!(complete_response.status(), Status::Ok);
        let completed_item = complete_response.into_json::<TodoItem>().unwrap();
        assert_eq!(completed_item.id, item_id);
        assert!(completed_item.completed);
        assert_eq!(completed_item.user_id.0, expected_user_uuid);

        let get_response = client.get(format!("/api/todos/{}", item_id)).dispatch();
        assert_eq!(get_response.status(), Status::Ok);
        let fetched_item = get_response.into_json::<TodoItem>().unwrap();
        assert!(fetched_item.completed);
        assert_eq!(fetched_item.user_id.0, expected_user_uuid);

        let non_existing_id = AppUuid::new_v4();
        let response_not_found = client.put(format!("/api/todos/{}/complete", non_existing_id)).dispatch();
        assert_eq!(response_not_found.status(), Status::NotFound);
    }

    #[test]
    fn test_search_todos() {
        let client = test_client();
        let expected_user_uuid = Uuid::parse_str(EXPECTED_GLOBAL_USER_UUID_STR).unwrap();

        client.post("/api/todos").header(ContentType::JSON).body(json!({ "description": "Learn Rust for GLOBAL user" }).to_string()).dispatch();
        client.post("/api/todos").header(ContentType::JSON).body(json!({ "description": "Learn Rocket for GLOBAL user" }).to_string()).dispatch();
        client.post("/api/todos").header(ContentType::JSON).body(json!({ "description": "Build an API for GLOBAL user" }).to_string()).dispatch();

        let response = client.get("/api/todos/search?description=learn").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let items = response.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items.len(), 2);
        for item in items {
            assert_eq!(item.user_id.0, expected_user_uuid);
            assert!(item.description.to_lowercase().contains("learn"));
        }

        let response_all = client.get("/api/todos/search").dispatch();
        assert_eq!(response_all.status(), Status::Ok);
        let items_all = response_all.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_all.len(), 3); // Assuming these are the only todos for this user
        for item in items_all {
            assert_eq!(item.user_id.0, expected_user_uuid);
        }
    }

    #[test]
    fn test_list_todos_by_status() {
        let client = test_client();
        let expected_user_uuid = Uuid::parse_str(EXPECTED_GLOBAL_USER_UUID_STR).unwrap();

        // Clear previous todos for this global user by fetching them to ensure count is clean for this test
        // This is a simple way to "reset" state for this test, specific to GLOBAL_USER_ID context
        // More robust test setup would involve clearing DB or using unique users per test.
        let initial_todos = client.get("/api/todos").dispatch().into_json::<Vec<TodoItem>>().unwrap_or_default();
        for _todo in initial_todos { // Prefixed with _
            // This part is tricky, as we don't have a delete endpoint yet.
            // For now, we'll assume tests run in an environment where this GLOBAL_USER_ID has no prior items,
            // or that prior items don't interfere with counts for *newly added* items in *this* test.
            // This highlights a limitation of the GLOBAL_USER_ID approach for test isolation.
        }


        let r1 = client.post("/api/todos").header(ContentType::JSON).body(json!({ "description": "Pending Task List" }).to_string()).dispatch();
        let _item1_id = r1.into_json::<TodoItem>().unwrap().id;
        client.post("/api/todos").header(ContentType::JSON).body(json!({ "description": "Another Pending List" }).to_string()).dispatch();

        let r3 = client.post("/api/todos").header(ContentType::JSON).body(json!({ "description": "Completed Task List" }).to_string()).dispatch();
        let item3_id = r3.into_json::<TodoItem>().unwrap().id;
        client.put(format!("/api/todos/{}/complete", item3_id)).dispatch();

        let response_true = client.get("/api/todos?completed=true").dispatch();
        assert_eq!(response_true.status(), Status::Ok);
        let items_true = response_true.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_true.len(), 1); // This might fail if other tests left completed items for GLOBAL_USER_ID
        for item in &items_true {
            assert!(item.completed);
            assert_eq!(item.user_id.0, expected_user_uuid);
        }
        if !items_true.is_empty() { // to prevent panic on items_true[0] if empty
             assert_eq!(items_true[0].description, "Completed Task List");
        }


        let response_false = client.get("/api/todos?completed=false").dispatch();
        assert_eq!(response_false.status(), Status::Ok);
        let items_false = response_false.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_false.len(), 2); // Similar to above, this assumes a clean slate for these 2 items.
        for item in items_false {
            assert!(!item.completed);
            assert_eq!(item.user_id.0, expected_user_uuid);
        }

        let response_all = client.get("/api/todos").dispatch();
        assert_eq!(response_all.status(), Status::Ok);
        let items_all = response_all.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_all.len(), 3); // Assumes only the 3 items from this test exist for GLOBAL_USER_ID
        for item in items_all {
            assert_eq!(item.user_id.0, expected_user_uuid);
        }
    }

    #[test]
    fn test_get_todos_count() {
        let client = test_client();
        let expected_user_uuid = Uuid::parse_str(EXPECTED_GLOBAL_USER_UUID_STR).unwrap();

        // To make this test more robust, we count before and after adding specific to this test.
        let initial_count = client.get("/api/todos/count").dispatch().into_json::<usize>().unwrap_or(0);

        let r1 = client.post("/api/todos").header(ContentType::JSON).body(json!({ "description": "Count Item 1" }).to_string()).dispatch();
        let item1 = r1.into_json::<TodoItem>().unwrap();
        assert_eq!(item1.user_id.0, expected_user_uuid);

        let r2 = client.post("/api/todos").header(ContentType::JSON).body(json!({ "description": "Count Item 2" }).to_string()).dispatch();
        let item2 = r2.into_json::<TodoItem>().unwrap();
        assert_eq!(item2.user_id.0, expected_user_uuid);


        let response_after_add = client.get("/api/todos/count").dispatch();
        assert_eq!(response_after_add.status(), Status::Ok);
        assert_eq!(response_after_add.into_json::<usize>().unwrap(), initial_count + 2);
    }

    #[test]
    fn test_get_todos_count_by_status() {
        let client = test_client();
        let expected_user_uuid = Uuid::parse_str(EXPECTED_GLOBAL_USER_UUID_STR).unwrap();

        // It's hard to isolate counts with a GLOBAL_USER_ID without a reset mechanism.
        // This test will assume it can add items and they'll be counted.
        // Consider running tests serially or clearing data if this becomes flaky.

        let desc_p1 = format!("Count Pending Status {}", Uuid::new_v4());
        let desc_p2 = format!("Count Pending Status {}", Uuid::new_v4());
        let desc_c1 = format!("Count Completed Status {}", Uuid::new_v4());

        client.post("/api/todos").header(ContentType::JSON).body(json!({ "description": desc_p1 }).to_string()).dispatch();
        client.post("/api/todos").header(ContentType::JSON).body(json!({ "description": desc_p2 }).to_string()).dispatch();
        let r3 = client.post("/api/todos").header(ContentType::JSON).body(json!({ "description": desc_c1 }).to_string()).dispatch();
        let item3_id = r3.into_json::<TodoItem>().unwrap().id;
        client.put(format!("/api/todos/{}/complete", item3_id)).dispatch();

        // Re-fetch all todos for GLOBAL_USER_ID to determine current counts accurately
        let all_todos_resp = client.get("/api/todos").dispatch();
        assert_eq!(all_todos_resp.status(), Status::Ok);
        let all_todos = all_todos_resp.into_json::<Vec<TodoItem>>().unwrap();

        let current_completed_count = all_todos.iter().filter(|item| item.completed && item.user_id.0 == expected_user_uuid).count();
        let current_pending_count = all_todos.iter().filter(|item| !item.completed && item.user_id.0 == expected_user_uuid).count();


        let response_true = client.get("/api/todos/count?completed=true").dispatch();
        assert_eq!(response_true.status(), Status::Ok);
        assert_eq!(response_true.into_json::<usize>().unwrap(), current_completed_count, "Completed count mismatch");

        let response_false = client.get("/api/todos/count?completed=false").dispatch();
        assert_eq!(response_false.status(), Status::Ok);
        assert_eq!(response_false.into_json::<usize>().unwrap(), current_pending_count, "Pending count mismatch");
    }
}
