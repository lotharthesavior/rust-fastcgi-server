use mime_guess::from_path;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufReader};
use log::{error, info};
use tokio::io;

pub fn is_allowed_static_file(file_path: &str) -> bool {
    let allowed_extensions = vec![".css", ".js", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico", ".html"];
    allowed_extensions.iter().any(|ext| file_path.ends_with(ext))
}

pub async fn serve_static_file(file_path: &str, mut socket: tokio::net::TcpStream) {
    let file = match File::open(file_path).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open file {}: {:?}", file_path, e);
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            if let Err(e) = socket.write_all(response.as_bytes()).await {
                eprintln!("Failed to send 404 response: {:?}", e);
            }
            if let Err(e) = socket.shutdown().await {
                eprintln!("Failed to shutdown socket after 404 response: {:?}", e);
            }
            return;
        }
    };

    let mime_type = from_path(file_path).first_or_octet_stream();
    let content_length = match file.metadata().await {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            error!("Failed to read metadata for file {}: {:?}", file_path, e);
            if let Err(e) = socket.shutdown().await {
                eprintln!("Failed to shutdown socket after metadata error: {:?}", e);
            }
            return;
        }
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        mime_type, content_length
    );

    if let Err(e) = socket.write_all(response.as_bytes()).await {
        error!("Failed to send response headers for file {}: {:?}", file_path, e);
        if let Err(e) = socket.shutdown().await {
            eprintln!("Failed to shutdown socket after header error: {:?}", e);
        }
        return;
    }

    let mut reader = BufReader::new(file);
    if let Err(e) = io::copy(&mut reader, &mut socket).await {
        error!("Failed to send file content: {:?}", e);
    }

    if let Err(e) = socket.flush().await {
        error!("Failed to flush socket: {:?}", e);
    }

    if let Err(e) = socket.shutdown().await {
        error!("Failed to shutdown socket: {:?}", e);
    }

    info!("Successfully served static file: {}", file_path);
}

#[cfg(test)]
mod tests {
    use crate::http::static_file::is_allowed_static_file;

    #[test]
    fn test_valid_static_files() {
        assert_eq!(is_allowed_static_file("test.css"), true);
        assert_eq!(is_allowed_static_file("test.js"), true);
        assert_eq!(is_allowed_static_file("index.png"), true);
        assert_eq!(is_allowed_static_file("index.jpg"), true);
        assert_eq!(is_allowed_static_file("index.jpeg"), true);
        assert_eq!(is_allowed_static_file("index.gif"), true);
        assert_eq!(is_allowed_static_file("index.svg"), true);
        assert_eq!(is_allowed_static_file("index.ico"), true);
        assert_eq!(is_allowed_static_file("index.html"), true);
    }

    #[test]
    fn test_not_accepted_as_static_file() {
        assert_eq!(is_allowed_static_file("index.php"), false);
    }
}
