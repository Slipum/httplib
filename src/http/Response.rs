//! HTTP Response utilities and builders.
//!
//! This module provides the `Response` structure and shorthand functions
//! like `text()` and `json()` to construct standard HTTP responses quickly.

use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;

pub (crate) struct Header {
    pub(crate) protocol: Option<String>, // Protocol like: HTTP/1.1 ...
    access_control_allow_origin: Option<String>, // Cors

    connection: Option<String>,
    content_encoding: Option<Vec<String>>,

    content_type: Option<Vec<String>>, // application/json, text/html or smth...

    date: Option<String>,
    etag: Option<String>,
    keep_alive: Option<HashMap<String, i32>>,
    last_modified: Option<String>,
    server: Option<String>,
    set_cookie: Option<HashMap<String, String>>, // hashmap Cookie
    transfer_encoding: Option<String>,
    vary: Option<Vec<String>>,
    x_backend_server: Option<String>,
    x_cache_info: Option<String>,
    x_kuma_revision: Option<i32>,
    x_frame_options: Option<String>,
}

pub struct Response {
    pub(crate) code: u16,
    pub(crate) phrase: Option<String>,

    pub(crate) header: Header,

    pub(crate) body: Option<String>,
}

impl Response {
    pub fn to_string(&self) -> String {
        let mut res = String::with_capacity(512);

        let protocol = self.header.protocol.as_deref().unwrap_or("HTTP/1.1");
        let phrase = self.phrase.as_deref().unwrap_or("");
        res.push_str(&format!("{} {} {}\r\n", protocol, self.code, phrase));

        if let Some(ref body) = self.body {
            res.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }

        let mut add_header = |name: &str, value: &Option<String>| {
            if let Some(v) = value {
                res.push_str(&format!("{}: {}\r\n", name, v));
            }
        };

        add_header("Access-Control-Allow-Origin", &self.header.access_control_allow_origin);
        add_header("Connection", &self.header.connection);
        add_header("Date", &self.header.date);
        add_header("Last-Modified", &self.header.last_modified);
        add_header("Server", &self.header.server);
        add_header("Transfer-Encoding", &self.header.transfer_encoding);
        add_header("X-Backend-Server", &self.header.x_backend_server);
        add_header("X-Cache-Info", &self.header.x_cache_info);
        add_header("X-Frame-Options", &self.header.x_frame_options);

        if let Some(ref etag) = self.header.etag {
            res.push_str(&format!("ETag: \"{}\"\r\n", etag));
        }

        if let Some(rev) = self.header.x_kuma_revision {
            res.push_str(&format!("X-kuma-revision: {}\r\n", rev));
        }

        let mut add_vec_header = |name: &str, vec_opt: &Option<Vec<String>>| {
            if let Some(vec) = vec_opt {
                res.push_str(&format!("{}: {}\r\n", name, vec.join(", ")));
            }
        };
        add_vec_header("Content-Encoding", &self.header.content_encoding);
        add_vec_header("Content-Type", &self.header.content_type);
        add_vec_header("Vary", &self.header.vary);

        if let Some(ref keep_alive) = self.header.keep_alive {
            let parts: Vec<String> = keep_alive
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            res.push_str(&format!("Keep-Alive: {}\r\n", parts.join(", ")));
        }

        if let Some(ref cookies) = self.header.set_cookie {
            for (key, value) in cookies {
                res.push_str(&format!("Set-Cookie: {}={}\r\n", key, value));
            }
        }

        res.push_str("\r\n");
        if let Some(ref body) = self.body {
            res.push_str(body);
        }

        res
    }

    pub fn set_phrase(mut self, phrase: &str) -> Response {
        self.phrase = Some(String::from(phrase));
        self
    }

    pub fn write(&self, mut stream: &TcpStream) {
        stream.write(self.to_string().as_bytes()).expect("failed to write to socket");
    }

    /// Use `HTTP/1.1`
    pub fn http1(mut self) -> Response {
        self.header.protocol = Some("HTTP/1.1".to_string());
        self
    }

    /// Use `HTTP/2`
    pub fn http2(mut self) -> Response {
        self.header.protocol = Some("HTTP/2".to_string());
        self
    }

    /// Use `HTTP/3`
    pub fn http3(mut self) -> Response {
        self.header.protocol = Some("HTTP/3".to_string());
        self
    }
}

pub fn text(code: u16, body: &str) -> Response {
    Response{
        code,
        phrase: None,
        header: Header {
            protocol: Some("HTTP/1.1".to_string()),
            access_control_allow_origin: None,
            connection: None,
            content_encoding: None,
            content_type: Some(vec!["text/html".to_string()]),
            date: None,
            etag: None,
            keep_alive: None,
            last_modified: None,
            server: None,
            set_cookie: None,
            transfer_encoding: None,
            vary: None,
            x_backend_server: None,
            x_cache_info: None,
            x_kuma_revision: None,
            x_frame_options: None,
        },
        body: Some(body.trim().to_string()),
    }
}

pub fn json(code: u16, body: &str) -> Response {
    Response{
        code,
        phrase: None,
        header: Header {
            protocol: Some("HTTP/1.1".to_string()),
            access_control_allow_origin: None,
            connection: None,
            content_encoding: None,
            content_type: Some(vec!["application/json".to_string(), "charset=utf-8".to_string()]),
            date: None,
            etag: None,
            keep_alive: None,
            last_modified: None,
            server: None,
            set_cookie: None,
            transfer_encoding: None,
            vary: None,
            x_backend_server: None,
            x_cache_info: None,
            x_kuma_revision: None,
            x_frame_options: None,
        },
        body: Some(body.trim().to_string()),
    }
}