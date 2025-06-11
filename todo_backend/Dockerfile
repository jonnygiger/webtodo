# todo_backend/Dockerfile

# --- Builder Stage ---
FROM rust:1.77-bullseye-slim AS builder
RUN rustup update stable

# Install diesel_cli and system dependencies for compiling pq-sys
RUN apt-get update && apt-get install -y libpq-dev     && rm -rf /var/lib/apt/lists/*
RUN cargo install diesel_cli --version 2.1.1 --no-default-features --features postgres

WORKDIR /usr/src/app

# Copy Cargo.toml, Cargo.lock, and src directory
COPY todo_backend/Cargo.toml todo_backend/Cargo.lock ./todo_backend/
COPY todo_backend/src ./todo_backend/src/
COPY todo_backend/migrations ./todo_backend/migrations/
COPY todo_backend/diesel.toml ./todo_backend/diesel.toml

# Create a dummy .env file for diesel schema generation if needed during build
# This won't be used at runtime if DATABASE_URL is provided by docker-compose
RUN echo "DATABASE_URL=postgres://dummyuser:dummypass@localhost:5432/dummy_db" > ./todo_backend/.env

# Update dependencies
RUN cd todo_backend && cargo update

# Build the application
# Separate target directory for dependencies caching
RUN --mount=type=cache,target=/usr/local/cargo/registry     --mount=type=cache,target=/usr/src/app/target     cd todo_backend && cargo build --release &&     cp target/release/todo_backend /usr/local/bin/todo_backend_server

# (Optional) Run migrations at build time if you want the schema baked into the image
# This requires the database to be accessible during the build, which is not typical for this stage.
# Usually, migrations are run by an init container or when the app starts.
# For now, we assume migrations will be run by the app on startup or by docker-compose.


# --- Final Stage ---
FROM debian:bullseye-slim

# Install runtime dependencies (libpq for PostgreSQL client)
RUN apt-get update && apt-get install -y libpq5 ca-certificates curl     && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary and diesel_cli from the builder stage
COPY --from=builder /usr/local/bin/todo_backend_server /usr/local/bin/todo_backend_server
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY todo_backend/entrypoint.sh /usr/local/bin/entrypoint.sh

# Copy static files if your application serves them
# COPY todo_backend/static /app/static

# Set working directory
WORKDIR /app

# Ensure entrypoint script is executable
RUN chmod +x /usr/local/bin/entrypoint.sh

# Expose the port the app runs on (Rocket default is 8000)
EXPOSE 8000

# Command to run the application
# The DATABASE_URL will be provided by docker-compose
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
