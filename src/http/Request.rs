use std::collections::HashMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
    DELETE,
    PUT,
    PATCH,
}

struct header {
    protocol: Option<String>, // Protocol like: HTTP/1.1 ...

    Host: Option<String>, // Url for host

    User_Agent: Option<String>, // Browser

    Accept: Option<Vec<String>>, // Type of request
    Accept_Language: Option<Vec<String>>,
    Accept_Encoding: Option<Vec<String>>,

    Sec_GPC: Option<i8>,
    Connection: Option<String>,
    Cookie: Option<HashMap<String, String>>, // Basic Cookie
    Upgrade_Insecure_Requests: Option<i8>,

    Sec_Fetch_Dest: Option<String>,
    Sec_Fetch_Mode: Option<String>,
    Sec_Fetch_Site: Option<String>,
    Sec_Fetch_User: Option<String>,

    DNT: Option<i8>,
    Priority: Option<String>,
}

pub struct Request {
    method: Option<Method>,
    route: Option<String>,

    header: Option<header>,

    body: Option<String>,
}

pub fn New() -> Request {
    Request{
        method: None,
        route: None,
        header: None,
        body: None,
    }
}

pub fn From(req: &[impl AsRef<str>]) -> Request {
    parse_request(&req).expect("Failed to parse http request")
}

impl Request {
    pub fn GetRoute(&self) -> String {
        String::from(self.route.as_ref().unwrap())
    }

    pub fn GetMethod(&self) -> Method {
        self.method.expect("Method not set")
    }

    pub fn GetBody(&self) -> String {
        String::from(self.body.as_ref().unwrap())
    }
}

fn parse_request(text: &[impl AsRef<str>]) -> Option<Request> {
    if text.is_empty() {
        return None;
    }

    let mut start = text[0].as_ref().split_whitespace();

    let method = start.next()?.to_string();
    let route = start.next()?.to_string();
    let protocol = start.next()?.to_string();



    let mut header = header{
        protocol: Some(protocol),
        Host: None,
        User_Agent: None,
        Accept: None,
        Accept_Language: None,
        Accept_Encoding: None,
        Sec_GPC: None,
        Connection: None,
        Cookie: None,
        Upgrade_Insecure_Requests: None,
        Sec_Fetch_Dest: None,
        Sec_Fetch_Mode: None,
        Sec_Fetch_Site: None,
        Sec_Fetch_User: None,
        DNT: None,
        Priority: None,
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
                "host" => header.Host = Some(value),
                "user-agent" => header.User_Agent = Some(value),
                "accept" => header.Accept = Some(value.split(',').map(|s| s.trim().to_string()).collect()),
                "accept-language" => header.Accept_Language = Some(value.split(',').map(|s| s.trim().to_string()).collect()),
                "accept-encoding" => header.Accept_Encoding = Some(value.split(',').map(|s| s.trim().to_string()).collect()),
                "sec-gpc" => header.Sec_GPC = Some(value.parse::<i8>().ok()?),
                "connection" => header.Connection = Some(value),
                "cookie" => header.Cookie = Some(parse_cookie(value)),
                "upgrade-insecure-requests" => header.Upgrade_Insecure_Requests = Some(value.parse::<i8>().ok()?),
                "sec-fetch-dest" => header.Sec_Fetch_Dest = Some(value),
                "sec-fetch-mode" => header.Sec_Fetch_Mode = Some(value),
                "sec-fetch-site" => header.Sec_Fetch_Site = Some(value),
                "sec-fetch-user" => header.Sec_Fetch_User = Some(value),
                "DNT" => header.DNT = Some(value.parse::<i8>().ok()?),
                "priority" => header.Priority = Some(value),
                _ => {}
            }
        }
    }

    Some(Request{
        method: Some(parse_method(method)),
        route: Some(route),
        header: Some(header),
        body: Some(text.last()?.as_ref().to_string()),
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