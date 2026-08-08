# httplib

Lightweight, synchronous, and multi-threaded Rust library for building fast HTTP servers from scratch.

[More documentation](https://docs.rs/httplib/latest/httplib/)

## Instalation 

- add to end `Cargo.toml`
```toml
[dependencies]
httplib = "1.2"
```
- or `bash` command
```bash
cargo add httplib
```

## Quick start

`main.rs`
```rust
use httplib::{Server, Router, Request, Response, Method, response};

fn handler_health(_request: &Request, _params: &[&str]) -> Response {
    response::text(200, "Hello from httplib!")
}

fn main() {
    let mut router = Router::new();
    router.add(Method::GET, "/hello", handler_health);

    let server = Server::new("localhost", 7878)
        .with_router(router)
        .enable_logger();

    server.start();
}
```

## Examples

[View examples](examples/)

### Server

```rust
fn main() {
    let router = build_router();

    let server = Server::new("localhost", 8000)
        .with_router(router) // setup router
        .enable_logger(); // add logger

    server.start();
}
```

### Router

```rust
fn build_router() -> Router {
    let mut router = Router::new();
    router
        .add(Method::GET, "/ping", handler_health)
        .add(Method::GET, "/hello", handler_hello)
        .add(Method::POST, "/user", handler_creat_user)
        .add(Method::GET, "/user/{id}", handler_get_user);
    router
}
```

> Supported method of http: 
```md
    GET,
    POST,
    DELETE,
    PUT,
    PATCH,
```

### Handles

> Basic 

- text `response::text(<status_code>, <text>)`
- json `response::json(<status_code>, <json_to_string>)`
- handle:
```rust
fn handle_name(_request: &Request, _params: &[&str]) -> Response {
    ...
}
```

### Route params

```rust
let name = _request.get_query("name").unwrap_or("");
let id = params.get(0).copied().unwrap_or("");
```

---

- Use text
```rust
fn handler_health(_request: &Request, _params: &[&str]) -> Response {
    response::text(200, "pong")
}
```

- Use JSON
```rust
fn handler_hello(_request: &Request, _params: &[&str]) -> Response {
    let id = params.get(0).copied().unwrap_or("");
    // let body = json!({ "user": id }).to_string(); // use lib `serde_json::json;`
    let body = format!("{{ \"user\": \"{id}\" }}"); // without json lib

    response::json(200, &body)
}
```

- Post with json
```rust

#[derive(Deserialize)]
struct CreateUserPayload {
    name: String,
}

fn handler_creat_user(_request: &Request, params: &[&str]) -> Response {
    match serde_json::from_str::<CreateUserPayload>(_request.get_body().as_str()) {
        Ok(user_data) => {
            let body = json!({ "message": format!("Create user with name: {}", user_data.name) }).to_string();
            response::json(201, &body)
        }
        Err(e) => {
            let body = json!({ "message": format!("Error with create user: {}", e) }).to_string();
            response::json(400, &body)
        }
    }
}
```

> use for json 
```toml
[dependencies]
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }
```
