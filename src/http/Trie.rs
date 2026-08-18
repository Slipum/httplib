use std::collections::HashMap;
use crate::{Method, Request, Response};

pub type Handler<T> = fn(&Request<T>, &[&str]) -> Response;


#[derive(Debug, Clone)]enum Segment {
    Static(String),
    Param(String),
    CatchAll(String),
}

fn parse_segment(s: &str) -> Segment {
    if s.starts_with('{') && s.ends_with('}') {
        let inner = &s[1..s.len() - 1];
        if let Some(rest_name) = inner.strip_prefix('*') {
            Segment::CatchAll(rest_name.to_string())
        } else {
            Segment::Param(inner.to_string())
        }
    } else if let Some(rest_name) = s.strip_prefix('*') {
        Segment::CatchAll(rest_name.to_string())
    } else if let Some(param_name) = s.strip_prefix(':') {
        Segment::Param(param_name.to_string())
    } else {
        Segment::Static(s.to_string())
    }
}

#[derive(Clone)]
pub struct Node<T> {
    handlers: HashMap<Method, Handler<T>>,
    static_children: HashMap<String, Node<T>>,
    param_child: Option<(String, Box<Node<T>>)>,
    catch_all_child: Option<(String, Box<Node<T>>)>,
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Self {
            handlers: HashMap::new(),
            static_children: HashMap::new(),
            param_child: None,
            catch_all_child: None,
        }
    }
}

impl<T> Node<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, method: Method, path_parts: &[&str], handler: Handler<T>) {
        if path_parts.is_empty() {
            self.handlers.insert(method, handler);
            return;
        }

        let segment = parse_segment(path_parts[0]);

        match segment {
            Segment::Static(name) => {
                self.static_children
                    .entry(name)
                    .or_insert_with(Node::new)
                    .insert(method, &path_parts[1..], handler);
            }
            Segment::Param(param_name) => {
                let (_, child) = self
                    .param_child
                    .get_or_insert_with(|| (param_name, Box::new(Node::new())));
                child.insert(method, &path_parts[1..], handler);
            }
            Segment::CatchAll(param_name) => {
                let (_, child) = self
                    .catch_all_child
                    .get_or_insert_with(|| (param_name, Box::new(Node::new())));
                child.handlers.insert(method, handler);
            }
        }
    }

    pub fn find(&self, method: Method, path_parts: &[&str]) -> Option<(Handler<T>, Vec<String>)> {
        let mut params = Vec::new();
        if let Some(handler) = self.match_path(method, path_parts, &mut params) {
            Some((handler, params))
        } else {
            None
        }
    }

    fn match_path(
        &self,
        method: Method,
        path_parts: &[&str],
        params: &mut Vec<String>,
    ) -> Option<Handler<T>> {
        if path_parts.is_empty() {
            return self.handlers.get(&method).copied();
        }

        let current_part = path_parts[0];

        if let Some(child) = self.static_children.get(current_part) {
            if let Some(handler) = child.match_path(method, &path_parts[1..], params) {
                return Some(handler);
            }
        }

        if let Some((_name, child)) = &self.param_child {
            params.push(current_part.to_string());
            if let Some(handler) = child.match_path(method, &path_parts[1..], params) {
                return Some(handler);
            }
            params.pop();
        }

        if let Some((_name, child)) = &self.catch_all_child {
            let rest_path = path_parts.join("/");
            params.push(rest_path);

            if let Some(handler) = child.handlers.get(&method) {
                return Some(*handler);
            }
            params.pop();
        }

        None
    }
}