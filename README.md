# x-http

An instant HTTP API testing suite - type-safe alternative to Postman/curl that integrates seamlessly with your Rust codebase.

## Features

- **Type-Safe Request Building**: Compile-time safety for your API tests
- **Chainable Assertions**: Fluent API for validating responses
- **JSON Path Queries**: Easy navigation of nested JSON structures
- **Works with `cargo test`**: Integrate API tests into your test suite
- **Interactive CLI Mode**: Quick ad-hoc testing with prompts
- **Configuration Files**: Store and version control your requests
- **Syntax Highlighting**: Beautiful colored output for JSON responses
- **Zero External Dependencies**: No need for separate GUI applications

## Installation

```bash
cargo install x-http
```

Or add to your `Cargo.toml`:

```toml
[dev-dependencies]
x-http = "0.1"
```

## Quick Start

### As a Library (Recommended)

```rust
use x_http::*;

#[test]
fn test_api_endpoint() {
    let response = Request::get("https://api.example.com/users/1")
        .header("Authorization", "Bearer YOUR_TOKEN")
        .send()
        .unwrap()
        .expect_status(200)
        .unwrap()
        .expect_json()
        .unwrap()
        .assert_field("id", 1)
        .unwrap()
        .assert_field("name", "John Doe")
        .unwrap();
}
```

### As a CLI Tool

#### Interactive Mode

```bash
# Start interactive session
x-http

# Or explicitly
x-http interactive
```

You'll be prompted for:
- HTTP method (GET, POST, PUT, etc.)
- URL
- Headers (optional)
- Request body (optional)

#### Quick Request

```bash
# Simple GET request
x-http request GET https://api.example.com/users

# POST with JSON body
x-http request POST https://api.example.com/users \
  -H "Content-Type:application/json" \
  --json \
  --body '{"name":"John","email":"john@example.com"}'

# With headers
x-http request GET https://api.example.com/protected \
  -H "Authorization:Bearer token123" \
  -H "Accept:application/json"
```

#### Configuration File

Create `x-http.toml`:

```toml
[variables]
BASE_URL = "https://api.example.com"
API_TOKEN = "your-token-here"

[[requests]]
name = "get-users"
method = "GET"
url = "{{BASE_URL}}/users"

[requests.headers]
Authorization = "Bearer {{API_TOKEN}}"

[[requests]]
name = "create-user"
method = "POST"
url = "{{BASE_URL}}/users"
json = true
body = """
{
  "name": "John Doe",
  "email": "john@example.com"
}
"""

[requests.headers]
Authorization = "Bearer {{API_TOKEN}}"
Content-Type = "application/json"
```

Run requests:

```bash
# Run all requests
x-http run

# Run specific request
x-http run --name get-users

# Use custom config file
x-http run --config my-requests.toml
```

## Usage Examples

### Making Requests

```rust
use x_http::*;

// GET request
let response = Request::get("https://api.example.com/users")
    .send()
    .unwrap();

// POST with JSON
let body = serde_json::json!({
    "name": "John",
    "email": "john@example.com"
});

let response = Request::post("https://api.example.com/users")
    .json(&body)
    .unwrap()
    .send()
    .unwrap();

// PUT with text body
let response = Request::put("https://api.example.com/note")
    .text("Note content")
    .send()
    .unwrap();

// DELETE request
let response = Request::delete("https://api.example.com/users/1")
    .send()
    .unwrap();

// PATCH request
let response = Request::patch("https://api.example.com/users/1")
    .json(&serde_json::json!({"email": "newemail@example.com"}))
    .unwrap()
    .send()
    .unwrap();
```

### Adding Headers

```rust
let response = Request::get("https://api.example.com/protected")
    .header("Authorization", "Bearer token123")
    .header("Accept", "application/json")
    .header("User-Agent", "my-app/1.0")
    .send()
    .unwrap();

// Or add multiple at once
let headers = vec![
    ("Authorization", "Bearer token123"),
    ("Accept", "application/json"),
];

let response = Request::get("https://api.example.com/protected")
    .headers(headers)
    .send()
    .unwrap();
```

### Query Parameters

```rust
let response = Request::get("https://api.example.com/search")
    .query("q", "rust programming")
    .query("limit", "10")
    .query("offset", "0")
    .send()
    .unwrap();

// Results in: https://api.example.com/search?q=rust+programming&limit=10&offset=0
```

### Timeouts and Redirects

```rust
use std::time::Duration;

let response = Request::get("https://api.example.com/slow")
    .timeout(Duration::from_secs(10))
    .send()
    .unwrap();

// Disable automatic redirects
let response = Request::get("https://api.example.com/redirect")
    .follow_redirects(false)
    .send()
    .unwrap();

// No timeout
let response = Request::get("https://api.example.com/endpoint")
    .no_timeout()
    .send()
    .unwrap();
```

### Status Code Assertions

```rust
// Expect specific status
let response = Request::get("https://api.example.com/users")
    .send()
    .unwrap()
    .expect_status(200)
    .unwrap();

// Expect any success status (2xx)
let response = Request::post("https://api.example.com/users")
    .json(&user_data)
    .unwrap()
    .send()
    .unwrap()
    .expect_success()
    .unwrap();

// Expect error status (4xx or 5xx)
let response = Request::get("https://api.example.com/nonexistent")
    .send()
    .unwrap()
    .expect_error()
    .unwrap();
```

### JSON Assertions

```rust
let response = Request::get("https://api.example.com/user/1")
    .send()
    .unwrap()
    .expect_json()  // Validates Content-Type and parseable JSON
    .unwrap()
    .assert_field("id", 1)
    .unwrap()
    .assert_field("name", "John Doe")
    .unwrap()
    .assert_field("email", "john@example.com")
    .unwrap()
    .assert_field_exists("created_at")
    .unwrap();

// Nested JSON paths
let response = Request::get("https://api.example.com/data")
    .send()
    .unwrap()
    .assert_field("user.profile.name", "John")
    .unwrap()
    .assert_field("settings.theme", "dark")
    .unwrap();

// Array access
let response = Request::get("https://api.example.com/users")
    .send()
    .unwrap()
    .assert_field("users[0].name", "Alice")
    .unwrap()
    .assert_field("users[1].id", 2)
    .unwrap()
    .assert_array_length("users", 10)
    .unwrap();
```

### Header Assertions

```rust
let response = Request::get("https://api.example.com/data")
    .send()
    .unwrap()
    .expect_header("content-type", "application/json")
    .unwrap()
    .expect_header("x-rate-limit-remaining", "99")
    .unwrap();

// Shorthand for content-type
let response = Request::get("https://api.example.com/data")
    .send()
    .unwrap()
    .expect_content_type("application/json")
    .unwrap();
```

### Body Content Assertions

```rust
let response = Request::get("https://example.com/page")
    .send()
    .unwrap()
    .expect_text()  // Validates body is valid UTF-8
    .unwrap()
    .expect_body_contains("<html>")
    .unwrap()
    .expect_body_contains("Welcome")
    .unwrap();
```

### Extracting Response Data

```rust
// Get status
let status = response.status();  // Returns u16
let status_code = response.status_code();  // Returns StatusCode

// Check status type
if response.is_success() {
    println!("Request succeeded!");
}

if response.is_error() {
    println!("Request failed!");
}

// Get headers
let content_type = response.header("content-type");
let all_headers = response.headers();

// Get body
let text = response.text().unwrap();
let bytes = response.body_bytes();

// Parse JSON
#[derive(Deserialize)]
struct User {
    id: u64,
    name: String,
}

let user: User = response.json().unwrap();

// Or as generic JSON value
let json: serde_json::Value = response.json_value().unwrap();

// Get request duration
let duration = response.duration();
println!("Request took {:?}", duration);
```

## Why x-http?

### vs Postman
- ✅ Version controlled (requests live in code)
- ✅ Works in CI/CD without GUI
- ✅ Type-safe with compile-time checks
- ✅ Integrates with `cargo test`
- ✅ No account/login required
- ✅ Free and open source

### vs curl
- ✅ Type-safe assertions
- ✅ Better error messages
- ✅ JSON path queries
- ✅ Chainable API
- ✅ Syntax highlighting
- ✅ Interactive mode

### vs other Rust HTTP clients
- ✅ Built for testing, not production
- ✅ Assertion-focused API
- ✅ CLI included
- ✅ Configuration file support
- ✅ Interactive mode

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of:

MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
