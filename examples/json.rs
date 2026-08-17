use httplib::{Server, Router, Request, Response, Method, response};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct CreateUserPayload {
    name: String,
}

fn handler_creat_user(_request: &Request, _params: &[&str]) -> Response {
    match serde_json::from_str::<CreateUserPayload>(_request.get_body()) {
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

fn build_router() -> Router {
    let mut router = Router::new();
    router
        .add(Method::POST, "/user", handler_creat_user);
    router
}

fn main() {
    let router = build_router();

    let server = Server::new("0.0.0.0", 7878)
        .with_router(router)
        .enable_logger();

    server.start();
}