
pub fn parse_http_request(request: &str) -> (&str, &str, &str) {
    let lines: Vec<&str> = request.split("\r\n").collect();
    let first_line = lines[0];
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");
    let version = parts.next().unwrap_or("");
    (method, path, version)
}
