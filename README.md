# webtodo

## Description

webtodo is a simple yet functional web application designed to help users manage their tasks effectively. It provides a clean interface for registering, logging in, and subsequently creating, viewing, and managing a personal todo list. The application is built with a Rust backend and a lightweight HTML/CSS/JavaScript frontend.

## Features

*   **User Authentication:**
    *   Secure user registration.
    *   User login and logout functionality.
*   **Todo Management:**
    *   Create new todo items with descriptions.
    *   View the list of existing todo items.
    *   Mark todo items as completed (assuming this is a standard feature, will verify if possible when checking API).
    *   Delete todo items.

## Tech Stack

### Backend
*   **Rust:** Main programming language for the backend.
*   **Rocket:** Web framework for Rust.
*   **Diesel:** ORM and query builder for PostgreSQL.
*   **PostgreSQL:** SQL database for data storage.
*   **Bcrypt:** For password hashing.

### Frontend
*   **HTML:** Structure of the web pages.
*   **CSS:** Styling of the web pages.
*   **JavaScript:** Client-side logic and interactivity.

### Containerization
*   **Docker:** For containerizing the application and database.
*   **Docker Compose:** For defining and running multi-container Docker applications.

## Prerequisites

To run this project locally, you will need the following installed:

*   **Docker:** [Link to Docker installation guide, e.g., https://docs.docker.com/get-docker/]
*   **Docker Compose:** (Usually included with Docker Desktop) [Link to Docker Compose installation guide, e.g., https://docs.docker.com/compose/install/]
*   **Git:** For cloning the repository.

## Getting Started

### Setup

1.  **Clone the repository:**
    ```bash
    git clone <repository_url>
    # Replace <repository_url> with the actual URL of this repository
    cd webtodo
    # Or your project's root directory name
    ```

2.  **Environment Variables:**
    The application uses a PostgreSQL database. The necessary environment variables for the database connection and application server are defined in the `docker-compose.yml` file. No manual `.env` file creation is required for default setup.
    *   `POSTGRES_USER`: myuser
    *   `POSTGRES_PASSWORD`: mypassword
    *   `POSTGRES_DB`: todo_db
    *   `DATABASE_URL`: postgres://myuser:mypassword@db:5432/todo_db (for the app service)
    *   `ROCKET_ADDRESS`: "0.0.0.0" (for the app service)

    These are configured to work together within the Docker Compose network.

### Running the Application

1.  **Build and Start Containers:**
    The `docker-compose.yml` file is configured to build the application image and run all services.
    ```bash
    docker-compose up -d --build
    ```
    *   The `--build` flag ensures images are built if they don't exist or if Dockerfiles have changed.
    *   The `-d` flag runs the containers in detached mode.

2.  **Database Migrations:**
    The `entrypoint.sh` script within the `app` container automatically runs database migrations upon startup. You can check the logs to confirm:
    ```bash
    docker-compose logs app
    ```

3.  **Access the Application:**
    Once the containers are up and running, you can access the web application in your browser at:
    [http://localhost:8000](http://localhost:8000)

4.  **Stopping the Application:**
    To stop the application and remove the containers:
    ```bash
    docker-compose down
    ```
    To stop without removing containers (so they can be restarted quickly):
    ```bash
    docker-compose stop
    ```

## Running Tests

The project includes a dedicated service in `docker-compose.yml` for running backend tests.

1.  **Ensure the database container is running:**
    The tests require the database service (`db`) to be active. If you haven't started all services, you can start the database specifically:
    ```bash
    docker-compose up -d db
    ```

2.  **Build the test image (if not already built):**
    The `test_runner` service in `docker-compose.yml` uses an image named `todo-builder`. This image needs to be built first. You can build it using a command like this, assuming your `todo_backend/Dockerfile` is structured to produce it (e.g., by tagging the result of a build stage):
    ```bash
    docker build -t todo-builder -f todo_backend/Dockerfile .
    ```
    *(Alternatively, if `todo-builder` is just the same image as the `app` service, `docker-compose build app` would suffice, and then `test_runner` would use the image built for `app` if `image: todo_rust_app` was specified or if `test_runner` also had a build section. Given `image: todo-builder`, an explicit build for that tag is safer to document).*

3.  **Run the tests:**
    Use the following command to execute the tests:
    ```bash
    docker-compose run --rm test_runner
    ```
    *   `run --rm`: Runs the `test_runner` service and removes the container after execution.
    *   The command `sh -c "diesel migration run && cargo test -- --test-threads=1"` within the `test_runner` service will first run database migrations and then execute the Rust tests.
    *   Test results will be displayed in your terminal.

## API Endpoints (Overview)

The application exposes the following API endpoints. All `/api/` routes require authentication.

### Authentication
*   **`POST /auth/register`**: Register a new user.
    *   Request Body: `{ "username": "your_username", "password": "your_password" }`
    *   Response: User information upon successful registration.
*   **`POST /auth/login`**: Log in an existing user.
    *   Request Body: `{ "username": "your_username", "password": "your_password" }`
    *   Response: Session token and username.
*   **`POST /auth/logout`**: Log out the current user.
    *   Clears the session cookie.

### Todo Items
*   **`POST /api/todos`**: Add a new todo item.
    *   Requires Authentication.
    *   Request Body: `{ "description": "Your todo description" }`
    *   Response: The created todo item.
*   **`GET /api/todos`**: List todo items for the authenticated user.
    *   Requires Authentication.
    *   Query Parameters (Optional):
        *   `description`: Filter by a search term in the description (e.g., `?description=meeting`).
        *   `completed`: Filter by completion status (e.g., `?completed=true` or `?completed=false`).
    *   Response: An array of todo items.
*   **`GET /api/todos/<item_id>`**: Get a specific todo item by its ID.
    *   Requires Authentication.
    *   Response: The requested todo item.
*   **`PUT /api/todos/<item_id>/complete`**: Mark a specific todo item as completed.
    *   Requires Authentication.
    *   Response: The updated todo item.
*   **`GET /api/todos/count`**: Get the count of todo items for the authenticated user.
    *   Requires Authentication.
    *   Query Parameters (Optional): Same as `GET /api/todos` for filtering the count.
    *   Response: A JSON object with the count (e.g., `{ "count": 5 }`).

*(Note: For detailed request/response schemas, please refer to the source code in `todo_backend/src/lib.rs` and `todo_backend/src/models.rs`.)*

## Project Structure

The repository is organized as follows:

*   **`.github/workflows/`**: Contains GitHub Actions workflow files (e.g., `rust.yml` for CI).
*   **`todo_backend/`**: Contains the Rust backend application.
    *   **`src/`**: Main source code for the backend (Rust files like `main.rs`, `lib.rs`, `db.rs`, `models.rs`, `schema.rs`).
    *   **`static/`**: Frontend static assets (HTML, CSS, JavaScript).
    *   **`migrations/`**: Diesel database migration files.
    *   **`tests/`**: Backend integration tests.
    *   **`Cargo.toml`**: Rust project manifest, defining dependencies and metadata.
    *   **`Dockerfile`**: Instructions for building the backend Docker image.
    *   **`.env`**: Example environment file (though actual env vars are set in `docker-compose.yml` for services).
    *   **`entrypoint.sh`**: Script run when the backend Docker container starts (runs migrations, starts server).
*   **`docker-compose.yml`**: Defines and configures the multi-container Docker application (backend app, database, test runner).
*   **`README.md`**: This file.

## Contributing

## License
