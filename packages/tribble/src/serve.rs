use std::net::SocketAddr;
use std::path::PathBuf;
use warp::Filter;

/// Serves the generated app from `.tribble/dist/`. This expects the app to already have been built.
pub async fn serve(dir: PathBuf, host: String, port: u16) {
    let dir = dir.join(".tribble/dist");
    // We actually don't have to worry about HTML file extensions at all
    let files = warp::any().and(warp::fs::dir(dir));
    // Parse `localhost` into `127.0.0.1` (picky Rust `std`)
    let host = if host == "localhost" {
        "127.0.0.1".to_string()
    } else {
        host
    };
    // Parse the host and port into an address
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    warp::serve(files).run(addr).await
}
