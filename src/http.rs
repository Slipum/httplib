use std::io::{BufRead, BufReader, Read};
use std::net::{TcpListener, TcpStream};

#[path = "http/Request.rs"]
pub mod request;
#[path = "http/Response.rs"]
pub mod response;
#[path = "http/Router.rs"]
pub mod router;

pub use request::{Request, Method};
pub use router::Router;
pub use response::Response;

#[derive(Clone, Default)]
pub struct Server {
    address: String,
    port: u16,

    router: Router,
    logger_enabled: bool,
}

impl Server {
    pub fn new(address: &str, port: u16) -> Self {
        Server {
            address: address.to_string(),
            port,
            router: Router::new(),
            logger_enabled: false,
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
            println!("Listening on {}", listener.local_addr().unwrap());
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
            None => http_route.as_str(),
        };

        let path_parts: Vec<&str> = clean_route
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        match self.router.find(req.get_method(), &path_parts) {
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
        let mut reader = BufReader::new(&stream);

        let mut lines: Vec<String> = Vec::new();
        let mut content_length: usize = 0;

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => return, // Соединение закрыто
                Ok(_) => {
                    if line == "\r\n" || line == "\n" {
                        break;
                    }

                    if line.to_lowercase().starts_with("content-length:") {
                        if let Some(val) = line.split(':').nth(1) {
                            content_length = val.trim().parse().unwrap_or(0);
                        }
                    }

                    lines.push(line.trim_end().to_string());
                }
                Err(e) => {
                    eprintln!("Failed to read from socket: {}", e);
                    return;
                }
            }
        }

        let mut body_bytes = vec![0u8; content_length];
        if content_length > 0 {
            if let Err(e) = reader.read_exact(&mut body_bytes) {
                eprintln!("Failed to read request body: {}", e);
                return;
            }
        }

        let body_str = String::from_utf8_lossy(&body_bytes).to_string();
        lines.push(body_str);

        let lines_ref: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        self.dispatch(lines_ref, &stream);
    }
}