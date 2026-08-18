use httplib::{Server, Router, Request, Response, Method, response};

fn handler_get_user(_request: &Request<()>, params: &[&str]) -> Response {
    let id = params.get(0).copied().unwrap_or("");
    let name = _request.get_query("name").unwrap_or("");
    let body = format!("{{ \"user\": \"{id}\", \"name\": \"{name}\" }}");

    response::json(200, &body)
}

fn handler_health(_request: &Request<()>, _params: &[&str]) -> Response {
    response::text(200, "pong")
}

fn handler_hello(_request: &Request<()>, _params: &[&str]) -> Response {
    let body = format!("{{ \"message\": \"hello\" }}");
    
    response::json(200, &body).http2().set_phrase("hello")
}

fn build_router() -> Router<()> {
    let mut router = Router::new();
    router
        .add(Method::GET, "/ping", handler_health)
        .add(Method::GET, "/hello", handler_hello)
        .add(Method::GET, "/user/{id}", handler_get_user);
    router
}

fn main() {
    let router = build_router();

    let server = Server::builder()
        .port(7878)
        .router(router)
        .enable_logger()
        .build();

    server.start();
}