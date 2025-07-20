# Codebase Streamlining

This document outlines the improvements made to the codebase to streamline its structure and improve its maintainability.

## 1. `services` Module

A new `services` module has been introduced to encapsulate the business logic of the application. This module contains sub-modules for `auth` and `todos`, which are responsible for handling authentication and todo item management, respectively. This change helps to separate the business logic from the web layer, making the code more modular, testable, and easier to maintain.

## 2. Database-Backed Authentication

The authentication system has been refactored to use a database-backed session store instead of an in-memory `DashMap`. A new `sessions` table has been added to the database to store session tokens, which makes the authentication system more robust and scalable.

## 3. Refactored Route Handlers

The route handlers have been refactored to use the new `services` module. This has made the route handlers thinner and more focused on handling HTTP requests and responses, as the business logic has been moved to the `services` module.

## 4. Improved Error Handling

A new `ServiceError` enum has been introduced in the `services` module to handle business logic errors. An implementation of `From<ServiceError> for ApiError` has been provided to automatically convert service errors into API errors, which helps to reduce boilerplate and centralize error handling.

## Error Handling

The error handling mechanism has been improved by using the `thiserror` and `anyhow` crates. The `ServiceError` enum now uses `thiserror` to derive the `Error` trait, which allows it to be used with the `?` operator for more concise error handling. The `anyhow` crate is used to provide more context to errors, making them easier to debug.

## Code Refactoring

The `build_todo_query` function has been refactored into a private helper function `_build_todo_query` to encapsulate the query building logic. This change improves the code's readability and maintainability by separating the query building logic from the `list_or_search_todos` and `get_todos_count` functions.
