use std::path::Path;
use std::path::PathBuf;

use axum::{routing, Router, Server};
use clap::Parser;
use tokio::fs::DirBuilder;

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

async fn datadir_init(data_dir: &Path) {
    match DirBuilder::new()
        .recursive(true)
        .create(data_dir.join("repositories/"))
        .await
    {
        Ok(_) => {}
        Err(e) => panic!(
            "Failed to create {}/repositories/. Error: {:?}",
            data_dir.to_string_lossy(),
            e
        ),
    }
}
async fn async_main() {
    let app = Router::new().route("/", routing::get(|| async { "Home Page" }));
    let args = std::sync::Arc::new(Args::parse());
    datadir_init(&args.data_dir).await;
    Server::bind(&"[::1]:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
