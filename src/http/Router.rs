use crate::http::{Method};
use crate::http::trie::{Handler, Node};

#[derive(Clone, Default)]
pub struct Router<T> {
    root: Node<T>,
}

impl<T> Router<T> {
    pub fn new() -> Self {
        Self { root: Node::new() }
    }

    pub fn add(&mut self, method: Method, path: &str, handler: Handler<T>) -> &mut Router<T> {
        let segments: Vec<&str> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        self.root.insert(method, &segments, handler);
        self
    }

    pub fn find(&self, method: Method, path_parts: &[&str]) -> Option<(Handler<T>, Vec<String>)> {
        self.root.find(method, path_parts)
    }
}
