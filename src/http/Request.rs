use std::collections::HashMap;

/// Supported HTTP methods for request routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    /// The GET method requests a representation of the specified resource.
    /// Requests using GET should only retrieve data.
    GET,

    /// The POST method submits an entity to the specified resource,
    /// often causing a change in state or side effects on the server.
    POST,

    /// The DELETE method deletes the specified resource.
    DELETE,

    /// The PUT method replaces all current representations of the target resource
    /// with the request payload.
    PUT,

    /// The PATCH method applies partial modifications to a resource.
    PATCH,

    /// The HEAD method asks for a response identical to a GET request,
    /// but without the response body.
    HEAD,

    /// The OPTIONS method describes the communication options for the target resource.
    OPTIONS,

    /// The CONNECT method establishes a tunnel to the server identified by the target resource.
    CONNECT,

    /// The TRACE method performs a message loop-back test along the path to the target resource.
    TRACE,
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

pub fn from(req: &[impl AsRef<str>]) -> Request {
    parse_request(&req).expect("Failed to parse http request")
}

impl Request {
    pub fn get_route(&self) -> &str {
        self.route.as_deref().unwrap_or_default()
    }

    pub fn get_method(&self) -> Option<Method> {
        self.method
    }

    pub fn get_body(&self) -> &str {
        self.body.as_deref().unwrap_or_default()
    }

    pub fn get_protocol(self) -> Option<String> {
        self.header?.protocol
    }

    /// Returns the parsed query parameter value by its key.
    ///
    /// Route: `/user?name=Nickname`
    /// # Examples
    /// ```rust
    /// use httplib::{response, Response, Request};
    ///
    /// fn handle_name(_request: &Request, _params: &[&str]) -> Response {
    ///     let name = _request.get_query("name").unwrap_or("Guest");
    ///
    ///     response::text(200, &format!("User with name: {name}").to_string())
    /// }
    /// ```
    pub fn get_query(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(|s| s.as_str())
    }
}

fn parse_request(text: &[impl AsRef<str>]) -> Option<Request> {
    if text.is_empty() {
        return None;
    }

    let mut start = text[0].as_ref().split_whitespace();

    let method_str = start.next()?.to_string();
    let raw_route = start.next()?.to_string();
    let protocol = start.next()?.to_string();

    let method = parse_method(method_str);

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
        method,
        route: Some(route),
        header: Some(header),
        body: Some(text.last()?.as_ref().to_string()),
        query: query_map,
    })
}

fn parse_method(method: impl AsRef<str>) -> Option<Method> {
    match method.as_ref() {
        "GET" => Some(Method::GET),
        "POST" => Some(Method::POST),
        "PUT" => Some(Method::PUT),
        "DELETE" => Some(Method::DELETE),
        "PATCH" => Some(Method::PATCH),
        "OPTIONS" => Some(Method::OPTIONS),
        "HEAD" => Some(Method::HEAD),
        "TRACE" => Some(Method::TRACE),
        "CONNECT" => Some(Method::CONNECT),
        _ => None,
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