use std::sync::Arc;
use httplib::{Server, Router, Request, Response, Method, response};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, FromRow};

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
}

#[derive(Serialize, FromRow)]
struct User {
    id: i64,
    name: String,
}

#[derive(Clone, Default)]
pub struct AppState {
    pub db: Option<SqlitePool>,
}

fn handler_get_users(request: &Request<Arc<AppState>>, _params: &[&str]) -> Response {
    println!("get_users");
    let rt = tokio::runtime::Runtime::new().unwrap();

    let pool = match request.state().db.as_ref() {
        Some(p) => p,
        None => return response::text(500, "Database pool not initialized"),
    };

    let result = rt.block_on(async {
        sqlx::query_as::<_, User>("SELECT id, name FROM users")
            .fetch_all(pool)
            .await
    });

    match result {
        Ok(users) => {
            let json = serde_json::to_string(&users).unwrap_or_default();
            response::json(200, &json)
        }
        Err(err) => {
            eprintln!("Database query error: {:?}", err);
            response::text(500, "Internal Server Error")
        }
    }
}

fn handler_create_user(request: &Request<Arc<AppState>>, _params: &[&str]) -> Response {
    let body = request.get_body();

    let payload: CreateUserRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(_) => return response::json(400, "{\"error\": \"Invalid JSON\"}"),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();

    let pool = match request.state().db.as_ref() {
        Some(p) => p,
        None => return response::text(500, "Database pool not initialized"),
    };

    let result = rt.block_on(async {
        sqlx::query("INSERT INTO users (name) VALUES (?)")
            .bind(&payload.name)
            .execute(pool)
            .await
    });

    match result {
        Ok(exec_res) => {
            let created_user = User {
                id: exec_res.last_insert_rowid(),
                name: payload.name,
            };
            let json = serde_json::to_string(&created_user).unwrap_or_default();
            response::json(201, &json)
        }
        Err(err) => {
            eprintln!("Database insert error: {:?}", err);
            response::text(500, "Internal Server Error")
        }
    }
}

fn handler_health(_request: &Request<Arc<AppState>>, _params: &[&str]) -> Response {
    response::text(200, "OK").set_phrase("OK")
}

fn build_router() -> Router::<Arc<AppState>> {
    let mut router = Router::<Arc<AppState>>::new();
    router.add(Method::GET, "/health", handler_health);
    router.add(Method::GET, "/users", handler_get_users);
    router.add(Method::POST, "/users", handler_create_user);
    router
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = "sqlite://app.db?mode=rwc";
    let pool = SqlitePool::connect(db_url).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
        );"
    )
        .execute(&pool)
        .await?;

    println!("Подключение к SQLite и миграция успешно выполнены");

    let state = Arc::new(AppState { db: Some(pool) });

    let server = Server::builder()
        .address("127.0.0.1")
        .port(7878)
        .router(build_router())
        .state(state)
        .enable_logger()
        .build();

    server.start();

    Ok(())
}