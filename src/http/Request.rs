use std::collections::HashMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
    DELETE,
    PUT,
    PATCH,
}

struct Header {
    protocol: Option<String>, // Protocol like: HTTP/1.1 ...

    host: Option<String>, // Url for host

    user_agent: Option<String>, // Browser

    accept: Option<Vec<String>>, // Type of request
    accept_language: Option<Vec<String>>,
    accept_encoding: Option<Vec<String>>,

    sec_gpc: Option<i8>,
    connection: Option<String>,
    cookie: Option<HashMap<String, String>>, // Basic Cookie
    upgrade_insecure_requests: Option<i8>,

    sec_fetch_dest: Option<String>,
    sec_fetch_mode: Option<String>,
    sec_fetch_site: Option<String>,
    sec_fetch_user: Option<String>,

    dnt: Option<i8>,
    priority: Option<String>,
}

pub struct Request {
    method: Option<Method>,
    route: Option<String>,

    header: Option<Header>,

    body: Option<String>,

    query: HashMap<String, String>,
}

pub fn new() -> Request {
    Request{
        method: None,
        route: None,
        header: None,
        body: None,
        query: HashMap::new(),
    }
}

pub fn from(req: &[impl AsRef<str>]) -> Request {
    parse_request(&req).expect("Failed to parse http request")
}

impl Request {
    pub fn get_route(&self) -> String {
        String::from(self.route.as_ref().unwrap())
    }

    pub fn get_method(&self) -> Method {
        self.method.expect("Method not set")
    }

    pub fn get_body(&self) -> String {
        String::from(self.body.as_ref().unwrap())
    }

    pub fn get_protocol(self) -> Option<String> {
        self.header?.protocol
    }

    pub fn get_query(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(|s| s.as_str())
    }
}

fn parse_request(text: &[impl AsRef<str>]) -> Option<Request> {
    if text.is_empty() {
        return None;
    }

    let mut start = text[0].as_ref().split_whitespace();

    let method = start.next()?.to_string();
    let raw_route = start.next()?.to_string();
    let protocol = start.next()?.to_string();

    let (route, query_map) = match raw_route.split_once('?') {
        Some((path, query_str)) => (path.to_string(), parse_query(query_str)),
        None => (raw_route, HashMap::new()),
    };

    let mut header = Header{
        protocol: Some(protocol),
        host: None,
        user_agent: None,
        accept: None,
        accept_language: None,
        accept_encoding: None,
        sec_gpc: None,
        connection: None,
        cookie: None,
        upgrade_insecure_requests: None,
        sec_fetch_dest: None,
        sec_fetch_mode: None,
        sec_fetch_site: None,
        sec_fetch_user: None,
        dnt: None,
        priority: None,
    };

    for l in &text[1..] {
        let line = l.as_ref().trim();

        if line.is_empty() {
            break;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim().to_string();

            match key.as_str() {
                "host" => header.host = Some(value),
                "user-agent" => header.user_agent = Some(value),
                "accept" => header.accept = Some(value.split(',').map(|s| s.trim().to_string()).collect()),
                "accept-language" => header.accept_language = Some(value.split(',').map(|s| s.trim().to_string()).collect()),
                "accept-encoding" => header.accept_encoding = Some(value.split(',').map(|s| s.trim().to_string()).collect()),
                "sec-gpc" => header.sec_gpc = Some(value.parse::<i8>().ok()?),
                "connection" => header.connection = Some(value),
                "cookie" => header.cookie = Some(parse_cookie(value)),
                "upgrade-insecure-requests" => header.upgrade_insecure_requests = Some(value.parse::<i8>().ok()?),
                "sec-fetch-dest" => header.sec_fetch_dest = Some(value),
                "sec-fetch-mode" => header.sec_fetch_mode = Some(value),
                "sec-fetch-site" => header.sec_fetch_site = Some(value),
                "sec-fetch-user" => header.sec_fetch_user = Some(value),
                "DNT" => header.dnt = Some(value.parse::<i8>().ok()?),
                "priority" => header.priority = Some(value),
                _ => {}
            }
        }
    }

    Some(Request{
        method: Some(parse_method(method)),
        route: Some(route),
        header: Some(header),
        body: Some(text.last()?.as_ref().to_string()),
        query: query_map,
    })
}

fn parse_method(method: impl AsRef<str>) -> Method {
    match method.as_ref() {
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "PATCH" => Method::PATCH,
        _ => Method::GET
    }
}

fn parse_cookie(cookies: impl AsRef<str>) -> HashMap<String, String> {
    let mut cookies_map = HashMap::new();

    for pair in cookies.as_ref().split("; ") {
        if let Some((c_key, c_value)) = pair.split_once('=') {
            cookies_map.insert(
                c_key.trim().to_string(),
                c_value.trim().to_string()
            );
        }
    }

    cookies_map
}

fn parse_query(query_str: &str) -> HashMap<String, String> {
    let mut query_map = HashMap::new();
    for pair in query_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            query_map.insert(
                key.trim().to_string(),
                value.trim().to_string()
            );
        }
    }
    query_map
}