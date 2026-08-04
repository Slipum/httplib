use std::net::TcpStream;
use crate::http::{Method, Request};
use crate::http::response::Response;

pub type Handler = fn(&Request, &[&str]) -> Response;

#[derive(Clone)]
struct Route {
    method: Method,
    segments: Vec<String>,
    handler: Handler,
}

#[derive(Clone, Default)]
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn add(&mut self, method: Method, path: &str, handler: Handler) -> &mut Router {
        let segments: Vec<String> = path
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        self.routes.push(Route {
            method,
            segments,
            handler,
        });
        self
    }

    /// Ищет маршрут, совпадающий по методу и сегментам пути.
    /// Сегмент вида "{id}" в определении маршрута считается параметром
    /// и совпадает с любым значением, которое возвращается в params.
    pub fn find(&self, method: Method, path_parts: &[&str]) -> Option<(Handler, Vec<String>)> {
        'route: for route in &self.routes {
            if route.method != method {
                continue;
            }
            if route.segments.len() != path_parts.len() {
                continue;
            }

            let mut params = Vec::new();
            for (seg, part) in route.segments.iter().zip(path_parts.iter()) {
                if seg.starts_with('{') && seg.ends_with('}') {
                    params.push(part.to_string());
                } else if seg != part {
                    continue 'route;
                }
            }

            return Some((route.handler, params));
        }
        None
    }
}