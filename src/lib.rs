//! # httplib
//!
//! Lightweight, synchronous, and multi-threaded Rust library for building fast HTTP servers from scratch.
//!
//! Built on top of the native standard library without the overhead of heavy async runtimes, it handles concurrency
//! by spawning a dedicated OS thread per connection. This architecture provides excellent predictable performance
//! for microservices, CLI tools, and embedded environments while keeping the code simple and free of `async/await` boilerplate.
//!
//! ## Quick Start
//! ```rust
//! use httplib::{Router, Method, Server, response};
//!
//! fn main() {
//!     let mut router = Router::new();
//!     router.add(Method::GET, "/", |_req, _params| {
//!         response::text(200, "Hello from httplib!")
//!     });
//!
//!     let server = Server::new("localhost", 8000)
//!         .with_router(router)
//!         .enable_logger();
//!
//!    server.start();
//!
//!     // Server will start listening on localhost:8000
//!     // Server::new("localhost", 8000).with_router(router).start();
//! }
//! ```

pub mod http;

#[doc(inline)]
pub use http::Server;
#[doc(inline)]
pub use http::Router;
#[doc(inline)]
pub use http::Request;
#[doc(inline)]
pub use http::Response;
#[doc(inline)]
pub use http::Method;

#[doc(inline)]
pub use http::response;

#[cfg(test)]
mod tests {
    use crate::{response};

    #[test]
    fn test_json_response_creation() {
        let res = response::json(200, "{\"status\":\"ok\"}");

        assert_eq!(res.code, 200);
        assert_eq!(res.body, Some("{\"status\":\"ok\"}".to_string()));
    }

    #[test]
    fn test_set_phrase() {
        let res = response::json(200, "body").set_phrase("Great Success");

        assert_eq!(res.phrase, Some("Great Success".to_string()));
    }

    #[test]
    fn test_http2_protocol() {
        let res = response::json(200, "body").http2();

        assert_eq!(res.header.protocol, Some("HTTP/2".to_string()));
    }

    #[test]
    fn test_http3_protocol() {
        let res = response::json(200, "body").http3();

        assert_eq!(res.header.protocol, Some("HTTP/3".to_string()));
    }

    #[test]
    fn test_json_response_error() {
        let res = response::json(500, "{}");

        assert_eq!(res.code, 500);
        assert_eq!(res.body, Some("{}".to_string()));
    }

    #[test]
    fn test_set_phrase_error() {
        let res = response::json(200, "body").set_phrase("");

        assert_eq!(res.phrase, Some("".to_string()));
    }
}
