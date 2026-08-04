use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;

struct Header {
    protocol: Option<String>, // Protocol like: HTTP/1.1 ...
    Access_Control_Allow_Origin: Option<String>, // Cors

    Connection: Option<String>,
    Content_Encoding: Option<Vec<String>>,

    Content_Type: Option<Vec<String>>, // application/json, text/html or smth...

    Date: Option<String>,
    ETag: Option<String>,
    Keep_Alive: Option<HashMap<String, i32>>,
    Last_Modified: Option<String>,
    Server: Option<String>,
    Set_Cookie: Option<HashMap<String, String>>, // hashmap Cookie
    Transfer_Encoding: Option<String>,
    Vary: Option<Vec<String>>,
    X_Backend_Server: Option<String>,
    X_Cache_Info: Option<String>,
    X_kuma_revision: Option<i32>,
    x_frame_options: Option<String>,
}

pub struct Response {
    code: i16,
    phrase: Option<String>,

    header: Header,

    body: Option<String>,
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

        add_header("Access-Control-Allow-Origin", &self.header.Access_Control_Allow_Origin);
        add_header("Connection", &self.header.Connection);
        add_header("Date", &self.header.Date);
        add_header("Last-Modified", &self.header.Last_Modified);
        add_header("Server", &self.header.Server);
        add_header("Transfer-Encoding", &self.header.Transfer_Encoding);
        add_header("X-Backend-Server", &self.header.X_Backend_Server);
        add_header("X-Cache-Info", &self.header.X_Cache_Info);
        add_header("X-Frame-Options", &self.header.x_frame_options);

        if let Some(ref etag) = self.header.ETag {
            res.push_str(&format!("ETag: \"{}\"\r\n", etag));
        }

        if let Some(rev) = self.header.X_kuma_revision {
            res.push_str(&format!("X-kuma-revision: {}\r\n", rev));
        }

        let mut add_vec_header = |name: &str, vec_opt: &Option<Vec<String>>| {
            if let Some(vec) = vec_opt {
                res.push_str(&format!("{}: {}\r\n", name, vec.join(", ")));
            }
        };
        add_vec_header("Content-Encoding", &self.header.Content_Encoding);
        add_vec_header("Content-Type", &self.header.Content_Type);
        add_vec_header("Vary", &self.header.Vary);

        if let Some(ref keep_alive) = self.header.Keep_Alive {
            let parts: Vec<String> = keep_alive
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            res.push_str(&format!("Keep-Alive: {}\r\n", parts.join(", ")));
        }

        if let Some(ref cookies) = self.header.Set_Cookie {
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

    pub fn set_phrase(&mut self, phrase: &str) {
        self.phrase = Some(String::from(phrase));
    }

    pub fn write(&self, mut stream: &TcpStream) {
        stream.write(self.to_string().as_bytes()).expect("failed to write to socket");
    }
}

pub fn text(code: i16, body: &str) -> Response {
    Response{
        code,
        phrase: None,
        header: Header {
            protocol: Some("HTTP/1.1".to_string()),
            Access_Control_Allow_Origin: None,
            Connection: None,
            Content_Encoding: None,
            Content_Type: Some(vec!["text/html".to_string()]),
            Date: None,
            ETag: None,
            Keep_Alive: None,
            Last_Modified: None,
            Server: None,
            Set_Cookie: None,
            Transfer_Encoding: None,
            Vary: None,
            X_Backend_Server: None,
            X_Cache_Info: None,
            X_kuma_revision: None,
            x_frame_options: None,
        },
        body: Some(body.trim().to_string()),
    }
}

pub fn json(code: i16, body: &str) -> Response {
    Response{
        code,
        phrase: None,
        header: Header {
            protocol: Some("HTTP/1.1".to_string()),
            Access_Control_Allow_Origin: None,
            Connection: None,
            Content_Encoding: None,
            Content_Type: Some(vec!["application/json".to_string(), "charset=utf-8".to_string()]),
            Date: None,
            ETag: None,
            Keep_Alive: None,
            Last_Modified: None,
            Server: None,
            Set_Cookie: None,
            Transfer_Encoding: None,
            Vary: None,
            X_Backend_Server: None,
            X_Cache_Info: None,
            X_kuma_revision: None,
            x_frame_options: None,
        },
        body: Some(body.trim().to_string()),
    }
}