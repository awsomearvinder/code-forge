use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use axum::Json;
use axum::{routing, Router, Server};
use clap::Parser;

use repositories::CommitLogReq;
use tokio::fs::DirBuilder;
use tokio_stream::StreamExt;

use tower_http::cors::{Any, CorsLayer};

mod entities;
mod frontend;
mod repositories;

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

async fn get_entries(path: &Path) -> Vec<OsString> {
    let mut entries =
        tokio_stream::wrappers::ReadDirStream::new(tokio::fs::read_dir(path).await.unwrap())
            .map(Result::unwrap);
    let mut entries_buff = Vec::new();
    while let Some(v) = entries.next().await {
        if !v.file_type().await.unwrap().is_dir() {
            eprintln!(
                "WARNING: found file ({}) in unexpected folder.",
                v.file_name().to_string_lossy()
            );
            continue;
        }
        entries_buff.push(v.file_name())
    }
    entries_buff
}

async fn async_main() {
    let args = std::sync::Arc::new(Args::parse());
    datadir_init(&args.data_dir).await;
    let f = std::sync::Arc::new(frontend::Frontend::new(args.clone()));
    let app =
        Router::new()
            .route("/", routing::get({
                let f = f.clone();
                move || async move { f.index().await }
            }))
            .route("/entities", routing::get({
                let f = f.clone();
                move || async move { f.entities().await }
            }))
            .route("/e/:name", routing::get({
                let f = f.clone();
                move |axum::extract::Path(name): axum::extract::Path<String>| async move { f.repositories(&name).await }
            }))
            .route("/r/:entity/:repo", routing::get({
                let f = f.clone();
                move |axum::extract::Path((entity, repo)): axum::extract::Path<(String, String)>, axum::extract::Query(req): axum::extract::Query<CommitLogReq>| async move { f.repository(&entity, &repo, &req).await }
            }))
            .route(
                "/api/entities",
                routing::get({
                    let args = args.clone();
                    move || async move { Json(entities::entities(&args).await) }
                }),
            )
            .route(
                "/api/:entity/repos",
                routing::get({
                    let args = args.clone();
                    move |axum::extract::Path(name): axum::extract::Path<String>| async move {
                        Json(entities::Entity::repos(&args, &name).await)
                    }
                }),
            )
            .route(
                "/api/:entity/:repo/commits",
                routing::get({
                    let args = args.clone();
                    move |axum::extract::Path((name, repo)): axum::extract::Path<(String, String)>,
                          axum::extract::Query(req): axum::extract::Query<CommitLogReq>| async move {
                        repositories::CommitLog::commit_log(&args, &name, &repo, &req).await.map(Json)
                    }
                }),
            )
            .layer(CorsLayer::new().allow_origin(Any));
    Server::bind(&"[::1]:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
