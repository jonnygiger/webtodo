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

echo "Starting application..."
docker-compose down --volumes
docker-compose up -d

echo "Running database migrations..."
docker-compose exec app diesel migration run
