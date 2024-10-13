
pub fn parse_http_request(request: &str) -> (&str, &str, &str) {
    let lines: Vec<&str> = request.split("\r\n").collect();
    let first_line = lines[0];
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");
    let version = parts.next().unwrap_or("");
    (method, path, version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_http_request() {
        let request = "GET /index.html HTTP/1.1\r\nHost: example.com\r\n";
        let (method, path, version) = parse_http_request(request);
        assert_eq!(method, "GET");
        assert_eq!(path, "/index.html");
        assert_eq!(version, "HTTP/1.1");
    }

    #[test]
    fn test_incomplete_http_request() {
        let request = "POST /submit";
        let (method, path, version) = parse_http_request(request);
        assert_eq!(method, "POST");
        assert_eq!(path, "/submit");
        assert_eq!(version, "");
    }

    #[test]
    fn test_empty_request() {
        let request = "";
        let (method, path, version) = parse_http_request(request);
        assert_eq!(method, "");
        assert_eq!(path, "");
        assert_eq!(version, "");
    }

    #[test]
    fn test_malformed_request_line() {
        let request = "INVALID\r\nSome-Header: value";
        let (method, path, version) = parse_http_request(request);
        assert_eq!(method, "INVALID");
        assert_eq!(path, "");
        assert_eq!(version, "");
    }
}
