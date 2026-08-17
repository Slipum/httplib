use httplib::{Server, Router, Request, Response, Method, response};

fn handler_health(_request: &Request, _params: &[&str]) -> Response {
    response::text(200, "Hello from httplib!")
}

fn main() {
    let mut router = Router::new();
    router.add(Method::GET, "/", handler_health);

    let server = Server::new("0.0.0.0", 7878)
        .with_router(router)
        .enable_logger();

    server.start();
}