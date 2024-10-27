use mime_guess::from_path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn is_allowed_static_file(file_path: &str) -> bool {
    let allowed_extensions = vec![".css", ".js", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico", ".html"];
    allowed_extensions.iter().any(|ext| file_path.ends_with(ext))
}

pub async fn serve_static_file(file_path: &str, socket: &mut tokio::net::TcpStream) {
    if let Ok(..) = File::open(file_path).await {
        let mut file = File::open(file_path).await.unwrap();
        let mime_type = from_path(file_path).first_or_octet_stream();

        let metadata = file.metadata().await.unwrap();
        let content_length = metadata.len();

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            mime_type, content_length
        );

        socket.write_all(response.as_bytes()).await.unwrap();

        let mut buffer = [0; 8192];
        loop {
            let n = file.read(&mut buffer).await.unwrap();
            if n == 0 {
                break;
            }
            socket.write_all(&buffer[..n]).await.unwrap();
        }

        socket.flush().await.unwrap();
    } else {
        let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string();
        socket.write_all(response.as_bytes()).await.unwrap();
    }
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
