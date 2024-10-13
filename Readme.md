
# Rust FastCGI Server

This is a simple FastCGI server written in Rust. It is intended to be used as a CGI server, to serve PHP webapps.

I really enjoyed writing this, and I hope you enjoy using it.

## Usage

To use this server, you need to have a FastCGI server running on your machine. You can use `php-fpm` for this purpose.

At this version, it only works with file socket (`unix:///path/to/php/socket.sock`), but I plan to add TCP support in the future.

You can run the development environment with the following command:

```bash
cargo run -- --base-path /path/to/laravel/public // or any other PHP app
```

To build:

```bash
cargo build --release
```

