use httplib::{Server, Router, Request, Response, Method, response};

fn handler_get_user(_request: &Request, params: &[&str]) -> Response {
    let id = params.get(0).copied().unwrap_or("");
    let body = format!("{{ \"user\": \"{id}\" }}");

    response::json(200, &body)
}

fn handler_health(_request: &Request, _params: &[&str]) -> Response {
    response::text(200, "pong")
}

fn handler_hello(_request: &Request, _params: &[&str]) -> Response {
    let body = format!("{{ \"message\": \"hello\" }}");

    response::json(200, &body)
}

// #[derive(Deserialize)]
// struct CreateUserPayload {
//     name: String,
// }
//
// fn handler_creat_user(_request: &Request, params: &[&str]) -> Response {
//     match serde_json::from_str::<CreateUserPayload>(_request.GetBody().as_str()) {
//         Ok(user_data) => {
//             let body = json!({ "message": format!("Create user with name: {}", user_data.name) }).to_string();
//             response::json(201, &body)
//         }
//         Err(e) => {
//             let body = json!({ "message": format!("Error with create user: {}", e) }).to_string();
//             response::json(400, &body)
//         }
//     }
// }

fn build_router() -> Router {
    let mut router = Router::new();
    router
        .add(Method::GET, "/ping", handler_health)
        .add(Method::GET, "/hello", handler_hello)
        // .add(Method::POST, "/user", handler_creat_user)
        .add(Method::GET, "/user/{id}", handler_get_user);
    router
}

fn main() {
    let router = build_router();

    let server = Server::new("0.0.0.0", 7878)
        .with_router(router)
        .enable_logger();

    server.start();
}