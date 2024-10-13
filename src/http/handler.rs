use std::net::SocketAddr;
use std::path::Path;
use tokio::net::{TcpStream, UnixStream};
use fastcgi_client::{Request, Client, Params};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::http;

pub async fn process(
    base_path: String,
    php_fpm_socket_path: String,
    socket: &mut TcpStream,
    addr: SocketAddr,
    port: u16,
) {
    let mut buffer = [0u8; 1024];

    let bytes_read = socket.read(&mut buffer).await.unwrap();

    if bytes_read <= 0 {
        return;
    }

    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let (method, path, _version) = http::request_parser::parse_http_request(&request_str);

    let (script_filename, is_static, script_name) = determine_script_filename_and_type(&base_path, path).await;

    println!("Request: {} {}", method, path);

    if is_static && Path::new(&script_filename).exists() {
        http::static_file::serve_static_file(&script_filename, socket).await;
    } else {
        let params: Params = Params::default()
            .request_method(method.to_string())
            .script_name(&script_name)
            .script_filename(&script_filename)
            .request_uri(path)
            .document_uri(path)
            .server_addr(addr.ip().to_string())
            .server_port(port)
            .content_type("")
            .content_length(0);

        let stream = UnixStream::connect(php_fpm_socket_path.clone()).await.unwrap();
        let client = Client::new(stream);
        let response = client.execute_once(Request::new(params, &mut io::empty())).await.unwrap();

        if let Some(stdout) = response.stdout {
            let (headers, body) = http::fastcgi::parse_fastcgi_response(stdout);

            let http_response = format!(
                "HTTP/1.1 200 OK\r\n{}\r\nContent-Length: {}\r\n\r\n{}",
                headers,
                body.len(),
                body
            );

            socket.write_all(http_response.as_bytes()).await.unwrap();
            socket.flush().await.unwrap();
        }
    }
}

async fn determine_script_filename_and_type(base_path: &str, path: &str) -> (String, bool, String) {
    let mut script_filename = format!("{}{}", base_path, path);
    let mut script_name = path.to_string();

    let is_static = http::static_file::is_allowed_static_file(path);

    if path == "/" {
        script_filename = format!("{}/index.php", base_path);
        script_name = "/index.php".to_string();
        if !Path::new(&script_filename).exists() {
            script_filename = format!("{}/index.html", base_path);
            script_name = "/index.html".to_string();
        }
    } else if !is_static && !Path::new(&script_filename).exists() {
        script_filename = format!("{}/index.php", base_path);
        script_name = "/index.php".to_string();
    }

    (script_filename, is_static, script_name)
}
