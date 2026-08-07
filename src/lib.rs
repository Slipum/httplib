pub mod http;
pub use http::{response, Method, Request, Response, Router, Server};

#[cfg(test)]
mod tests {
    use crate::{response};

    #[test]
    fn test_json_response_creation() {
        let res = response::json(200, "{\"status\":\"ok\"}");

        assert_eq!(res.code, 200);
        assert_eq!(res.body, Some("{\"status\":\"ok\"}".to_string()));
    }

    #[test]
    fn test_set_phrase() {
        let res = response::json(200, "body").set_phrase("Great Success");

        assert_eq!(res.phrase, Some("Great Success".to_string()));
    }

    #[test]
    fn test_http2_protocol() {
        let res = response::json(200, "body").http2();

        assert_eq!(res.header.protocol, Some("HTTP/2".to_string()));
    }

    #[test]
    fn test_http3_protocol() {
        let res = response::json(200, "body").http3();

        assert_eq!(res.header.protocol, Some("HTTP/3".to_string()));
    }

    #[test]
    fn test_json_response_error() {
        let res = response::json(500, "{}");

        assert_eq!(res.code, 500);
        assert_eq!(res.body, Some("{}".to_string()));
    }

    #[test]
    fn test_set_phrase_error() {
        let res = response::json(200, "body").set_phrase("");

        assert_eq!(res.phrase, Some("".to_string()));
    }
}
