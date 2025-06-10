#!/bin/sh
set -e

# Run database migrations
echo "Running database migrations..."
/usr/local/bin/diesel migration run

# Start the main application
echo "Starting application..."
exec /usr/local/bin/todo_backend_server
