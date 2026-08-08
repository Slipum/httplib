//! # httplib
//!
//! Lightweight, synchronous, and multi-threaded Rust library for building fast HTTP servers from scratch.
//!
//! Built on top of the native standard library without the overhead of heavy async runtimes, it handles concurrency
//! by spawning a dedicated OS thread per connection. This architecture provides excellent predictable performance
//! for microservices, CLI tools, and embedded environments while keeping the code simple and free of `async/await` boilerplate.
//!
//! ## Quick Start
//! ```rust,no_run
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
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;
    use crate::{Server, Method, Router, response};

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

    #[test]
    fn test_huge_content_length_does_not_hang_or_alloc() {
        let mut router = Router::new();
        router.add(Method::POST, "/upload", |_req, _params| {
            response::text(200, "OK")
        });

        let port = 18080;
        let server = Server::new("127.0.0.1", port).with_router(router);

        thread::spawn(move || {
            server.start();
        });

        thread::sleep(Duration::from_millis(100));

        let mut stream =
            TcpStream::connect(("127.0.0.1", port)).expect("Не удалось подключиться к серверу");

        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("Failed to set read timeout");

        let raw_request = "POST /upload HTTP/1.1\r\n\
                           Host: 127.0.0.1\r\n\
                           Content-Type: application/octet-stream\r\n\
                           Content-Length: 4000000000\r\n\
                           Connection: close\r\n\
                           \r\n";

        stream
            .write_all(raw_request.as_bytes())
            .expect("Ошибка отправки заголовков");

        let mut response = String::new();
        let read_result = stream.read_to_string(&mut response);

        match read_result {
            Ok(_) => {
                assert!(
                    response.contains("413") || response.contains("400"),
                    "Сервер вернул неожиданный статус-код. Ответ сервера:\n{}",
                    response
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                panic!("FAIL: Сервер завис в ожидании 4 ГБ тела запроса и превысил таймаут в 2 секунды!");
            }
            Err(e) => {
                println!("Сервер корректно сбросил соединение: {:?}", e.kind());
            }
        }
    }
}
