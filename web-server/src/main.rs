use std::path::PathBuf;

use axum::{routing, Router, Server};
use clap::Parser;

/// Webserver component for the code forge.
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    data_dir: PathBuf,
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    let app = Router::new().route("/", routing::get(|| async { "Home Page" }));
    let _args = std::sync::Arc::new(Args::parse());
    Server::bind(&"[::1]:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
