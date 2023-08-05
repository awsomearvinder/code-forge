use axum::{routing, Router, Server};

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .unwrap();
    rt.block_on(async_main());
}
async fn async_main() {
    let app = Router::new().route("/", routing::get(|| async { "Home Page" }));
    Server::bind(&"[::1]:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
