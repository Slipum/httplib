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

    pub fn find(&self, method: Method, path_parts: &[&str]) -> Option<(Handler, Vec<String>)> {
        for route in &self.routes {
            if route.method != method {
                continue;
            }

            if let Some(params) = parse_params(&route.segments, path_parts) {
                return Some((route.handler, params));
            }
        }
        None
    }
}

fn parse_params(route_segments: &[String], path_parts: &[&str]) -> Option<Vec<String>> {
    if route_segments.len() != path_parts.len() {
        return None;
    }

    let mut params = Vec::new();

    for (seg, part) in route_segments.iter().zip(path_parts.iter()) {
        if seg.starts_with('{') && seg.ends_with('}') {
            params.push(part.to_string());
        } else if seg != part {
            return None;
        }

        
    }

    Some(params)
}