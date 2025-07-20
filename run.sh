#!/bin/bash
set -e

# Check for Docker
if ! [ -x "$(command -v docker)" ]; then
  echo 'Error: docker is not installed.' >&2
  exit 1
fi

# Check for Docker Compose
if ! [ -x "$(command -v docker-compose)" ]; then
  echo 'Error: docker-compose is not installed.' >&2
  exit 1
fi

echo "Starting database..."
docker-compose up -d db

echo "Waiting for database to be ready..."
while ! docker-compose exec db pg_isready -U myuser -d todo_db > /dev/null 2>&1; do
  sleep 1
done

echo "Running database migrations..."
docker-compose run --rm app diesel migration run

echo "Starting application..."
docker-compose up -d app
