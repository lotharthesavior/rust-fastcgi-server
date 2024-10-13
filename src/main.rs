use tokio::{net::TcpListener};
use clap::Parser;

mod http {
    pub mod static_file;
    pub mod request_parser;
    pub mod fastcgi;
    pub mod handler;
}

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    base_path: String,

    #[clap(short = 's', long = "socket", default_value = "/run/php/php-fpm.sock")]
    socket_path: String,

    #[clap(short, long, default_value = "8080")]
    port: u16,

    #[clap(long, default_value = "0.0.0.0")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let addr = format!("{}:{}", cli.host, cli.port);
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (mut socket, addr) = listener.accept().await?;
        let base_path = cli.base_path.clone();
        let port = cli.port.clone();
        let socket_path = cli.socket_path.clone().to_string();

        tokio::spawn(async move {
            http::handler::process(
                base_path,
                socket_path,
                &mut socket,
                addr,
                port,
            ).await;
        });
    }
}
