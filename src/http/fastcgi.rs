use std::str;

pub fn parse_fastcgi_response(response: Vec<u8>) -> (String, String) {
    let response_str = match String::from_utf8(response.clone()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error converting response to UTF-8: {:?}", e);
            String::from_utf8_lossy(&response).to_string()
        }
    };

    let parts: Vec<&str> = response_str.splitn(2, "\r\n\r\n").collect();

    let (headers, body) = if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        (String::new(), response_str)
    };

    (headers, body)
}
