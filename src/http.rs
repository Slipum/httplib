use std::io::{BufRead, BufReader, ErrorKind, Read};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

#[path = "http/Request.rs"]
mod request;
#[path = "http/Response.rs"]
pub mod response;
#[path = "http/Router.rs"]
mod router;
#[path = "http/Trie.rs"]
mod trie;

pub use request::{Request, Method};
pub use router::Router;
pub use response::Response;

#[derive(Clone, Default)]
pub struct Server {
    address: String,
    port: u16,

    router: Router,
    logger_enabled: bool,
    max_body_size: usize,
}

impl Server {
    pub fn with_max_body_size(mut self, size: usize) -> Self {
        self.max_body_size = size;
        self
    }

    pub fn new(address: &str, port: u16) -> Self {
        Server {
            address: address.to_string(),
            port,
            router: Router::new(),
            logger_enabled: false,
            max_body_size: 10 * 1024 * 1024,
        }
    }

    pub fn enable_logger(mut self) -> Self {
        self.logger_enabled = true;
        self
    }

    pub fn with_router(mut self, router: Router) -> Self {
        self.router = router;
        self
    }

    pub fn start(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.address, self.port)).unwrap();
        if self.logger_enabled {
            println!("Listening on http://{}", listener.local_addr().unwrap());
        }

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let server = self.clone();
                    std::thread::spawn(move || server.client(stream));
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }

    fn dispatch(&self, text: Vec<&str>, stream: &TcpStream) {
        let req: Request = request::from(&text);
        if self.logger_enabled {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            println!("[LOGGER] {} | {:?} {}", now, req.get_method(), req.get_route());
        }

        let http_route = req.get_route();

        let clean_route = match http_route.split_once('?') {
            Some((path, _query_string)) => {
                path
            }
            None => http_route,
        };

        let path_parts: Vec<&str> = clean_route
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let method = match req.get_method() {
            Some(met) => met,
            None => {
                response::text(405, "Method Not Allowed").write(&stream);
                return;
            }
        };

        match self.router.find(method, &path_parts) {
            Some((handler, params)) => {
                let params_refs: Vec<&str> = params.iter().map(|s| s.as_str()).collect();
                handler(&req, &params_refs).write(&stream);
            }
            None => {
                response::text(404, "Not Found").write(stream);
            }
        }
    }

    fn client(&self, stream: TcpStream) {
        if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(15))) {
            eprintln!("Failed to set read timeout: {}", e);
            return;
        }

        let mut reader = BufReader::new(&stream);

        loop {
            let mut lines: Vec<String> = Vec::new();
            let mut content_length: Option<usize> = None;
            let mut close_requested = false;
            let mut keep_alive_requested = false;
            let mut is_http_1_0 = false;

            const MAX_HEADER_LINES: usize = 100;
            const MAX_LINE_LENGTH: usize = 8192;

            loop {
                if lines.len() >= MAX_HEADER_LINES {
                    response::text(431, "Request Header Fields Too Large").write(&stream);
                    return;
                }

                let mut line = String::new();
                let bytes_read = match reader.by_ref().take(MAX_LINE_LENGTH as u64).read_line(&mut line) {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        if (e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock) && lines.is_empty() {
                            return;
                        }

                        if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock {
                            response::text(408, "Request Timeout").write(&stream);
                        } else {
                            eprintln!("Failed to read from socket: {}", e);
                        }
                        return;
                    }
                };

                if bytes_read >= MAX_LINE_LENGTH && !line.ends_with('\n') {
                    response::text(431, "Request Header Fields Too Large").write(&stream);
                    return;
                }

                if line == "\r\n" || line == "\n" {
                    break;
                }

                let lower_line = line.to_lowercase();

                if lines.is_empty() && lower_line.contains("http/1.0") {
                    is_http_1_0 = true;
                }

                if lower_line.starts_with("content-length:") {
                    if let Some(val) = line.split(':').nth(1) {
                        match val.trim().parse::<usize>() {
                            Ok(len) => content_length = Some(len),
                            Err(_) => {
                                response::text(400, "Bad Request: Invalid Content-Length").write(&stream);
                                return;
                            }
                        }
                    }
                }

                if lower_line.starts_with("connection:") {
                    if let Some(val) = line.split(':').nth(1) {
                        let conn_val = val.trim().to_lowercase();
                        if conn_val.contains("close") {
                            close_requested = true;
                        }
                        if conn_val.contains("keep-alive") {
                            keep_alive_requested = true;
                        }
                    }
                }

                lines.push(line.trim_end().to_string());
            }

            let should_close = if is_http_1_0 {
                !keep_alive_requested
            } else {
                close_requested
            };

            let content_length = content_length.unwrap_or(0);

            if content_length > self.max_body_size {
                response::text(413, "Payload Too Large").write(&stream);
                return;
            }

            let mut body_bytes = Vec::with_capacity(content_length);
            if content_length > 0 {
                if let Err(e) = reader.by_ref().take(content_length as u64).read_to_end(&mut body_bytes) {
                    if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock {
                        response::text(408, "Request Timeout: Slow Body").write(&stream);
                    } else {
                        eprintln!("Failed to read request body: {}", e);
                        response::text(400, "Bad Request: Incomplete Body").write(&stream);
                    }
                    return;
                }
            }

            let body_str = String::from_utf8_lossy(&body_bytes).to_string();
            lines.push(body_str);

            let lines_ref: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
            self.dispatch(lines_ref, &stream);

            if should_close {
                break;
            }
        }
    }
}