# httplib

Lightweight, synchronous, and multi-threaded Rust library for building fast, secure HTTP servers from scratch.

Built on top of the native Rust standard library without the overhead of heavy `async` runtimes, `httplib` handles concurrency by spawning a dedicated OS thread per connection. It provides predictable performance, low memory footprint, and robust built-in security features while keeping the codebase simple and free of `async/await` boilerplate.

[Full Documentation on docs.rs](https://docs.rs/httplib/latest/httplib/)

---

## Installation

Add `httplib` to your `Cargo.toml`:

```toml
[dependencies]
httplib = "*"

```

Or run via Cargo CLI:

```bash
cargo add httplib
```

---

## Quick Start

```rust
use httplib::{response, Method, Request, Response, Router, Server};

fn handler_hello(_request: &Request<()>, _params: &[&str]) -> Response {
    response::text(200, "Hello from httplib!")
}

fn main() {
    let mut router = Router::<()>::new();
    router.add(Method::GET, "/", handler_hello);

    let server = Server::builder()
        .port(7878)
        .router(router)
        .enable_logger()
        .build();

    server.start();
}
```

---

## Key Features & Advantages

* **Zero Async Overhead:** Pure synchronous `thread-per-connection` model using standard library primitives. No complex async runtimes or hidden memory bloat.
* **Persistent Connections (HTTP Keep-Alive):** Full support for HTTP/1.1 and HTTP/1.0 connection reuse, reducing TCP handshake overhead with automatic idle timeouts and `Connection: close` handling.
* **Fast Radix/Trie Tree Router:** $O(\text{path depth})$ route matching powered by a segment-based Trie tree (similar to `matchit`). Replaces slow $O(N)$ linear route scanning.
* **Advanced Route Patterns:** Supports static segments, path parameters (`/user/{id}` or `/user/:id`), and wildcards / catch-all tails (`/static/{*filepath}`).
* **Comprehensive HTTP Methods Support:** Full support for `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`, `CONNECT`, and `TRACE`.
* **Allocation DoS Defense:** Prevents OOM attacks by strictly validating `Content-Length` and capping buffer allocations before reading request bodies.
* **Slowloris & Header Flooding Protection:** Built-in read timeouts (`408 Request Timeout`) and strict boundaries on header size and count (`431 Request Header Fields Too Large`).
* **Zero-Dependency Core:** Highly portable and ideal for microservices, CLI applications, embedded targets, and edge environments.

---

## Router Features & Path Matching

`httplib` utilizes a Trie-based router with strict priority rules: **Static > Parameters > Wildcards**.

```rust
fn build_router() -> Router::<()> {
    let mut router = Router::new();
    
    router
        // Static route
        .add(Method::GET, "/ping", handler_ping)
        
        // Named parameter route (/user/123 -> params[0] = "123")
        .add(Method::GET, "/user/{id}", handler_get_user)
        
        // Multiple parameter route
        .add(Method::PUT, "/posts/{category}/{id}", handler_update_post)
        
        // Catch-all / Wildcard route (/static/css/style.css -> params[0] = "css/style.css")
        .add(Method::GET, "/static/{*filepath}", handler_static_files);

    router
}
```

### Supported HTTP Methods

| Method | Enum Variant | Typical Usage |
| --- | --- | --- |
| `GET` | `Method::GET` | Retrieve resource representation |
| `POST` | `Method::POST` | Create resource or process payload |
| `PUT` | `Method::PUT` | Replace resource completely |
| `DELETE` | `Method::DELETE` | Remove specified resource |
| `PATCH` | `Method::PATCH` | Apply partial modifications |
| `HEAD` | `Method::HEAD` | Fetch headers identical to `GET` without body |
| `OPTIONS` | `Method::OPTIONS` | Describe target communication options (CORS) |
| `CONNECT` | `Method::CONNECT` | Establish tunnel connection |
| `TRACE` | `Method::TRACE` | Perform loop-back test |

---

## Handling Requests & Responses

### Query Parameters & Path Params

```rust
fn handler_get_user(request: &Request<()>, params: &[&str]) -> Response {
    // Extract path parameter (e.g. /user/{id})
    let user_id = params.get(0).copied().unwrap_or("0");

    // Extract query parameter (e.g. /user/123?format=full)
    let format = request.get_query("format").unwrap_or("standard");

    let response_body = format!(r#"{{"user_id": "{user_id}", "format": "{format}"}}"#);
    response::json(200, &response_body)
}
```

### Working with JSON Payloads

```rust
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct CreateUserPayload {
    name: String,
}

fn handler_create_user(request: &Request<()>, _params: &[&str]) -> Response {
    match serde_json::from_str::<CreateUserPayload>(request.get_body().as_str()) {
        Ok(payload) => {
            let body = json!({
                "status": "success",
                "message": format!("Created user: {}", payload.name)
            }).to_string();

            response::json(201, &body)
        }
        Err(e) => {
            let body = json!({
                "error": "Invalid JSON payload",
                "details": e.to_string()
            }).to_string();

            response::json(400, &body)
        }
    }
}
```

---

## Built-In Security & Protection Mechanisms

`httplib` includes built-in safeguards against common HTTP vulnerability vectors:

* **Resource Exhaustion (OOM) Protection:** Incoming `Content-Length` headers are validated against `max_body_size` (10 MB by default) prior to heap allocation. Initial vector capacities are capped at small bounds (`64 KB`) and reallocated safely on demand during streaming.
* **Header DoS Guard:** Enforces strict limits on maximum header count (100 lines) and individual line length (8 KB) to prevent unbounded memory consumption from malicious request headers.
* **Slowloris Defense:** Configurable read timeouts (`408 Request Timeout`) on underlying TCP streams automatically drop stalled or intentionally slow connection attempts.
* **Strict Header Validation:** Non-numeric or malformed `Content-Length` values are rejected with `400 Bad Request` to prevent Request Smuggling vulnerabilities.

## Examples

Explore full server examples in the [`examples/`](examples) directory.