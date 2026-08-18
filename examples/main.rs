use httplib::{Server, Router, Request, Response, Method, response};

// If not need state, in Request use `()`
fn handler_health(_request: &Request<()>, _params: &[&str]) -> Response {
    response::text(200, "Hello from httplib!")
}

fn main() {
    let mut router = Router::new();
    router.add(Method::GET, "/", handler_health);

    // Use `()` for default server without state
    let server = Server::builder()
        .address("0.0.0.0")
        .port(7878)
        .router(router)
        .enable_logger()
        .build();

    server.start();
}