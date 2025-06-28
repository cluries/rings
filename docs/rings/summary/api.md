# src/web/api.rs

This file defines the standard API response structure (`Out`) used across the `rings` project, particularly for web services built with `axum`. It provides a consistent format for success, error, and debug information.

## Type Aliases

### `OutAny`
An alias for `Out<serde_json::Value>`, used for API responses where the data payload can be any JSON value.

## Structs

### `Out<T: Serialize>`
A generic struct representing the standard API response format.

#### Fields
*   `code: String`: A string representing the status code of the response.
*   `message: Option<String>`: An optional human-readable message associated with the response.
*   `data: Option<T>`: An optional data payload, where `T` is a serializable type.
*   `debug: Option<Debug>`: Optional debug information.
*   `profile: Option<Profile>`: Optional profiling information.

#### Methods
*   `new(code: LayoutedC, message: Option<String>, data: Option<T>) -> Self`: Creates a new `Out` instance with a code, optional message, and optional data.
*   `only_code(code: LayoutedC) -> Self`: Creates an `Out` instance with only a code.
*   `code_message(code: LayoutedC, message: &str) -> Self`: Creates an `Out` instance with a code and an optional message.
*   `ok(data: T) -> Self`: Creates a successful `Out` instance with data.
*   `set_debug(&mut self, debug: Debug)`: Sets the debug information for the response.
*   `set_profile(&mut self, profile: Profile)`: Sets the profiling information for the response.

### `Debug`
Contains debug-related information, primarily a map of other key-value pairs.

#### Methods
*   `new() -> Debug`: Creates a new empty `Debug` instance.
*   `add_other(&mut self, key: &str, value: &str) -> &mut Debug`: Adds a key-value pair to the debug information.

### `Profile`
Currently an empty struct, intended for profiling information.

## Implementations

### `impl<T: Serialize> From<Except> for Out<T>`
Converts an `Except` error type into an `Out` response.

### `impl<T: Serialize> From<Erx> for Out<T>`
Converts an `Erx` error type into an `Out` response, including original error code in debug info.

### `impl<T: Serialize> From<Option<T>> for Out<T>`
Converts an `Option<T>` into an `Out` response. `Some(data)` becomes `Out::ok(data)`, `None` becomes an `Except::Unknown` error.

### `impl<T: Serialize, E: ToString> From<Result<T, E>> for Out<T>`
Converts a `Result<T, E>` into an `Out` response. `Ok(v)` becomes `Out::ok(v)`, `Err(e)` becomes an `Except::Unknown` error with the error message.

### `impl<T: Serialize> axum::response::IntoResponse for Out<T>`
Enables `Out<T>` to be directly used as an `axum` response. It serializes the `Out` struct to JSON, sets the `Content-Type` header to `application/json`, and handles potential JSON serialization errors by returning an internal server error.
