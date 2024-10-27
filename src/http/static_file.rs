use mime_guess::from_path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn is_allowed_static_file(file_path: &str) -> bool {
    let allowed_extensions = vec![".css", ".js", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico", ".html"];
    allowed_extensions.iter().any(|ext| file_path.ends_with(ext))
}

pub async fn serve_static_file(file_path: &str, socket: &mut tokio::net::TcpStream) {
    let file = match File::open(file_path).await {
        Ok(f) => f,
        Err(_) => {
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            if let Err(e) = socket.write_all(response.as_bytes()).await {
                eprintln!("Failed to send 404 response: {:?}", e);
            }
            return;
        }
    };

    let mime_type = from_path(file_path).first_or_octet_stream();
    let content_length = match file.metadata().await {
        Ok(metadata) => metadata.len(),
        Err(_) => {
            eprintln!("Failed to read file metadata.");
            return;
        }
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        mime_type, content_length
    );

    if socket.write_all(response.as_bytes()).await.is_err() {
        eprintln!("Failed to send response headers.");
        return;
    }

    let mut buffer = [0; 8192];
    let mut file = file;
    loop {
        let n = match file.read(&mut buffer).await {
            Ok(0) => break, // EOF reached
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error reading file: {:?}", e);
                return;
            }
        };

        if socket.write_all(&buffer[..n]).await.is_err() {
            eprintln!("Failed to send file content.");
            return;
        }
    }

    if socket.flush().await.is_err() {
        eprintln!("Failed to flush socket.");
    }

    let _ = socket.shutdown().await;
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
