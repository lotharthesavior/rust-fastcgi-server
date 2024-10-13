
pub fn parse_fastcgi_response(response: Vec<u8>) -> (String, String) {
    let response_str = String::from_utf8_lossy(&response);
    let parts: Vec<&str> = response_str.splitn(2, "\r\n\r\n").collect();

    let (headers, body) = if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        (String::new(), response_str.to_string())
    };

    (headers, body)
}
