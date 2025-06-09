#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use serde_json::json;
    use todo_backend::*; // Import all public items from todo_backend crate
    // Uuid from uuid crate is still needed if we want to call .to_string() on a new one for a path.
    // AppUuid is used in TodoItem.id.
    // For test Uuid generation for paths, AppUuid::new_v4().to_string() or Uuid::new_v4().to_string() are fine.
    // The key is that TodoItem.id is AppUuid, and FromParam works for AppUuid.
    // Removed RwLock and DashMap as they are not directly used in tests.

    // Helper function to create a test client
    fn test_client() -> Client {
        // Use the rocket_instance from the library crate
        let rocket_instance = todo_backend::rocket_instance();
        Client::tracked(rocket_instance).expect("valid rocket instance")
    }

    #[test]
    fn test_add_todo() {
        let client = test_client();
        let description = "Test todo item - add";
        let response = client.post("/todos")
            .header(ContentType::JSON)
            .body(json!({ "description": description }).to_string())
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let item = response.into_json::<TodoItem>().unwrap();
        assert_eq!(item.description, description);
        assert!(!item.completed);
        // Check if the item is in storage (optional, direct check not straightforward here)
        // For now, we trust the add_todo function's logic and response.
        // A get_todo test will verify storage.
    }

    #[test]
    fn test_get_todo() {
        let client = test_client();
        let description = "Test todo item - get";

        // Add an item first
        let add_response = client.post("/todos")
            .header(ContentType::JSON)
            .body(json!({ "description": description }).to_string())
            .dispatch();
        assert_eq!(add_response.status(), Status::Ok);
        let added_item = add_response.into_json::<TodoItem>().unwrap();
        let item_id = added_item.id;

        // Test retrieving the existing item
        let response = client.get(format!("/todos/{}", item_id)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let fetched_item = response.into_json::<TodoItem>().unwrap();
        assert_eq!(fetched_item.id, item_id);
        assert_eq!(fetched_item.description, description);

        // Test retrieving a non-existing item
        let non_existing_id = AppUuid::new_v4(); // Use AppUuid here
        // AppUuid will deref to Uuid, which implements Display for the format macro
        let response_not_found = client.get(format!("/todos/{}", non_existing_id)).dispatch();
        assert_eq!(response_not_found.status(), Status::NotFound);
    }

    #[test]
    fn test_complete_todo() {
        let client = test_client();
        let description = "Test todo item - complete";

        // Add an item first
        let add_response = client.post("/todos")
            .header(ContentType::JSON)
            .body(json!({ "description": description }).to_string())
            .dispatch();
        assert_eq!(add_response.status(), Status::Ok);
        let added_item = add_response.into_json::<TodoItem>().unwrap();
        let item_id = added_item.id;

        // Mark the item as complete
        let complete_response = client.put(format!("/todos/{}/complete", item_id)).dispatch();
        assert_eq!(complete_response.status(), Status::Ok);
        let completed_item = complete_response.into_json::<TodoItem>().unwrap();
        assert_eq!(completed_item.id, item_id);
        assert!(completed_item.completed);

        // Verify it's completed by getting it again
        let get_response = client.get(format!("/todos/{}", item_id)).dispatch();
        assert_eq!(get_response.status(), Status::Ok);
        let fetched_item = get_response.into_json::<TodoItem>().unwrap();
        assert!(fetched_item.completed);

        // Test completing a non-existing item
        let non_existing_id = AppUuid::new_v4(); // Use AppUuid here
        // AppUuid will deref to Uuid, which implements Display for the format macro
        let response_not_found = client.put(format!("/todos/{}/complete", non_existing_id)).dispatch();
        assert_eq!(response_not_found.status(), Status::NotFound);
    }

    #[test]
    fn test_search_todos() {
        let client = test_client();
        // Add some items
        client.post("/todos").header(ContentType::JSON).body(json!({ "description": "Learn Rust" }).to_string()).dispatch();
        client.post("/todos").header(ContentType::JSON).body(json!({ "description": "Learn Rocket" }).to_string()).dispatch();
        client.post("/todos").header(ContentType::JSON).body(json!({ "description": "Build an API" }).to_string()).dispatch();

        // Test search with a query (case-insensitive)
        let response = client.get("/todos/search?description=learn").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let items = response.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|item| item.description == "Learn Rust"));
        assert!(items.iter().any(|item| item.description == "Learn Rocket"));

        // Test search with a query that matches part of a word
        let response_partial = client.get("/todos/search?description=rock").dispatch();
        assert_eq!(response_partial.status(), Status::Ok);
        let items_partial = response_partial.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_partial.len(), 1);
        assert_eq!(items_partial[0].description, "Learn Rocket");

        // Test search with a query that matches nothing
        let response_none = client.get("/todos/search?description=python").dispatch();
        assert_eq!(response_none.status(), Status::Ok);
        let items_none = response_none.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_none.len(), 0);

        // Test search with no query (should return all)
        let response_all = client.get("/todos/search").dispatch();
        assert_eq!(response_all.status(), Status::Ok);
        let items_all = response_all.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_all.len(), 3);
    }

    #[test]
    fn test_list_todos_by_status() {
        let client = test_client();
        // Add items
        let r1 = client.post("/todos").header(ContentType::JSON).body(json!({ "description": "Pending Task" }).to_string()).dispatch();
        let _item1_id = r1.into_json::<TodoItem>().unwrap().id; // item1_id is AppUuid
        client.post("/todos").header(ContentType::JSON).body(json!({ "description": "Another Pending" }).to_string()).dispatch();

        let r3 = client.post("/todos").header(ContentType::JSON).body(json!({ "description": "Completed Task" }).to_string()).dispatch();
        let item3_id = r3.into_json::<TodoItem>().unwrap().id; // item3_id is AppUuid
        client.put(format!("/todos/{}/complete", item3_id)).dispatch();


        // Test list with completed=true
        let response_true = client.get("/todos?completed=true").dispatch();
        assert_eq!(response_true.status(), Status::Ok);
        let items_true = response_true.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_true.len(), 1);
        assert!(items_true.iter().all(|item| item.completed));
        assert_eq!(items_true[0].description, "Completed Task");

        // Test list with completed=false
        let response_false = client.get("/todos?completed=false").dispatch();
        assert_eq!(response_false.status(), Status::Ok);
        let items_false = response_false.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_false.len(), 2);
        assert!(items_false.iter().all(|item| !item.completed));

        // Test list with no status (should return all)
        let response_all = client.get("/todos").dispatch();
        assert_eq!(response_all.status(), Status::Ok);
        let items_all = response_all.into_json::<Vec<TodoItem>>().unwrap();
        assert_eq!(items_all.len(), 3);
    }

    #[test]
    fn test_get_todos_count() {
        let client = test_client();
        // Initial count
        let response = client.get("/todos/count").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_json::<usize>().unwrap(), 0);

        // Add items and check count
        client.post("/todos").header(ContentType::JSON).body(json!({ "description": "Item 1" }).to_string()).dispatch();
        client.post("/todos").header(ContentType::JSON).body(json!({ "description": "Item 2" }).to_string()).dispatch();

        let response_after_add = client.get("/todos/count").dispatch();
        assert_eq!(response_after_add.status(), Status::Ok);
        assert_eq!(response_after_add.into_json::<usize>().unwrap(), 2);
    }

    #[test]
    fn test_get_todos_count_by_status() {
        let client = test_client();
        // Add items
        let r1 = client.post("/todos").header(ContentType::JSON).body(json!({ "description": "Count Pending 1" }).to_string()).dispatch();
        let _item1_id = r1.into_json::<TodoItem>().unwrap().id;
        client.post("/todos").header(ContentType::JSON).body(json!({ "description": "Count Pending 2" }).to_string()).dispatch();

        let r3 = client.post("/todos").header(ContentType::JSON).body(json!({ "description": "Count Completed 1" }).to_string()).dispatch();
        let item3_id = r3.into_json::<TodoItem>().unwrap().id;
        client.put(format!("/todos/{}/complete", item3_id)).dispatch();

        // Test count with completed=true
        let response_true = client.get("/todos/count?completed=true").dispatch();
        assert_eq!(response_true.status(), Status::Ok);
        assert_eq!(response_true.into_json::<usize>().unwrap(), 1);

        // Test count with completed=false
        let response_false = client.get("/todos/count?completed=false").dispatch();
        assert_eq!(response_false.status(), Status::Ok);
        assert_eq!(response_false.into_json::<usize>().unwrap(), 2);
    }
}
