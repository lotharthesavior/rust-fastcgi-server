use std::net::SocketAddr;
use std::path::Path;
use anyhow::anyhow;
use tokio::net::{TcpStream, UnixStream};
use fastcgi_client::{Request, Client, Params};
use fastcgi_client::conn::ShortConn;
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use crate::http;

pub async fn process(
    base_path: String,
    php_fpm_socket_path: String,
    mut socket: TcpStream,
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
        let params: Params<'_> = Params::default()
            .request_method(method.to_string())
            .script_name(&script_name)
            .script_filename(&script_filename)
            .request_uri(path)
            .document_uri(path)
            .server_addr(addr.ip().to_string())
            .server_port(port)
            .content_type("")
            .content_length(0);

        if php_fpm_socket_path.starts_with('/') {
            match UnixStream::connect(&php_fpm_socket_path).await {
                Ok(unix_stream) => {
                    let client = Client::new(unix_stream);
                    execute_fastcgi_request(client, params, socket).await.unwrap();
                }
                Err(e) => {
                    eprintln!("Failed to connect to PHP-FPM Unix socket: {:?}", e);

                    let response = "HTTP/1.1 500 Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                        eprintln!("Failed to send 500 response: {:?}", e);
                    }

                    if let Err(e) = socket.shutdown().await {
                        eprintln!("Failed to shutdown socket after header error: {:?}", e);
                    }
                    return;
                }
            }
        } else {
            match TcpStream::connect(php_fpm_socket_path).await {
                Ok(tcp_stream) => {
                    let client = Client::new(tcp_stream);
                    execute_fastcgi_request(client, params, socket).await.unwrap();
                }
                Err(e) => {
                    eprintln!("Failed to connect to PHP-FPM TCP port: {:?}", e);

                    let response = "HTTP/1.1 500 Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                        eprintln!("Failed to send 500 response: {:?}", e);
                    }

                    if let Err(e) = socket.shutdown().await {
                        eprintln!("Failed to shutdown socket after header error: {:?}", e);
                    }
                    return;
                }
            }
        }
    }
}

async fn execute_fastcgi_request<S>(
    client: Client<S, ShortConn>,
    params: Params<'_>,
    mut socket: TcpStream,
) -> Result<(), anyhow::Error>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let result = async {
        let mut empty_reader = io::empty();
        let request = Request::new(params, &mut empty_reader);

        let response = client.execute_once(request).await.map_err(|e| {
            eprintln!("Failed to execute FastCGI request: {:?}", e);
            anyhow!("FastCGI execution failed")
        })?;

        if let Some(stdout) = response.stdout {
            let (headers, body) = http::fastcgi::parse_fastcgi_response(stdout);

            let http_response = format!(
                "HTTP/1.1 200 OK\r\n{}\r\nContent-Length: {}\r\nConnection: close\r\nX-Jacked: Everything is worth it if the soul is not small.\r\n\r\n{}",
                headers,
                body.len(),
                body
            );

            socket.write_all(http_response.as_bytes()).await.map_err(|e| {
                eprintln!("Failed to write response to socket: {:?}", e);
                anyhow!("Failed to write response")
            })?;

            socket.flush().await.map_err(|e| {
                eprintln!("Failed to flush socket: {:?}", e);
                anyhow!("Failed to flush socket")
            })?;

            Ok::<(), anyhow::Error>(())
        } else {
            eprintln!("No stdout in FastCGI response");
            anyhow::bail!("No stdout in FastCGI response");
        }
    }.await;

    match result {
        Ok(_) => {
            if let Err(e) = socket.shutdown().await {
                eprintln!("Failed to shutdown socket after successful request: {:?}", e);
            }
        }
        Err(e) => {
            if let Err(shutdown_err) = socket.shutdown().await {
                eprintln!(
                    "Failed to shutdown socket for FastCGI request after error: {:?}",
                    shutdown_err
                );
            }

            let response =
                "HTTP/1.1 500 Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            if let Err(e) = socket.write_all(response.as_bytes()).await {
                eprintln!("Failed to send 500 response: {:?}", e);
            }
            eprintln!("Error executing FastCGI request: {:?}", e);
        }
    }

    Ok(())
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
