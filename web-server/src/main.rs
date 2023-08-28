use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use axum::http::StatusCode;

use axum::Json;
use axum::{routing, Router, Server};
use clap::Parser;

use tokio::fs::DirBuilder;
use tokio_stream::StreamExt;

use tower_http::cors::{Any, CorsLayer};

/// Webserver component for the code forge.
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    data_dir: PathBuf,
}

#[derive(serde::Serialize)]
struct Entities {
    entities: Vec<Entity>,
}

#[derive(serde::Serialize)]
struct Repos {
    repos: Vec<Repo>,
}

#[derive(serde::Serialize)]
struct Repo {
    name: String,
}

#[derive(serde::Serialize)]
struct CommitLog {
    commits: Vec<Commit>,
}

#[derive(serde::Serialize)]
struct Commit {
    message_header: String,
    message_body: String,
}

#[derive(serde::Serialize)]
struct Entity {
    name: String,
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

async fn repo_page(args: &Args, entity: &str, name: &str) -> Result<Json<CommitLog>, StatusCode> {
    let repo =
        git2::Repository::open_bare(args.data_dir.join(format!("repositories/{entity}/{name}")))
            .map_err(|e| match e.code() {
                git2::ErrorCode::NotFound => StatusCode::NOT_FOUND,
                e => {
                    panic!("Couldn't open repo {entity}/{name}, got unexpected error: {e:?}")
                }
            })?;
    let mut walk = repo.revwalk().unwrap();
    walk.push(repo.head().unwrap().peel_to_commit().unwrap().id())
        .unwrap();
    let messages: Vec<Commit> = walk
        .take(10)
        .map(|oid| {
            let commit = repo.find_commit(oid.unwrap()).unwrap();
            let message = commit.message().unwrap_or("(empty commit message)");
            let [header, body @ ..]: &[&str] = &message.split('\n').collect::<Vec<_>>()[..] else { unreachable!() }; // body is empty in the case where there's no new line
            Commit {
                message_header: header.to_string(),
                message_body: body.join("\n"),
            }
        })
        .collect();
    Ok(Json(CommitLog { commits: messages }))
}

async fn entity_page(args: &Args, entity_name: &str) -> Json<Repos> {
    let repo_entry_links = get_entries(&args.data_dir.join(format!("repositories/{entity_name}")))
        .await
        .into_iter()
        .map(|i| Repo {
            name: i.to_str().unwrap().to_owned(),
        })
        .collect();
    Json(Repos {
        repos: repo_entry_links,
    })
}

async fn entities(args: &Args) -> Json<Entities> {
    let entities: Vec<Entity> = get_entries(dbg!(&args.data_dir.join("repositories/")))
        .await
        .into_iter()
        .map(|i| Entity {
            name: i.to_str().unwrap().to_owned(),
        })
        .collect();
    Json(Entities { entities })
}

async fn async_main() {
    let args = std::sync::Arc::new(Args::parse());
    datadir_init(&args.data_dir).await;
    let app =
        Router::new()
            .route(
                "/api/entities",
                routing::get({
                    let args = args.clone();
                    move || async move { entities(&args).await }
                }),
            )
            .route(
                "/api/:entity/repos",
                routing::get({
                    let args = args.clone();
                    move |axum::extract::Path(name): axum::extract::Path<String>| async move {
                        entity_page(&args, &name).await
                    }
                }),
            )
            .route(
                "/api/:entity/:repo/commits",
                routing::get({
                    let args = args.clone();
                    move |axum::extract::Path((name, repo)): axum::extract::Path<(
                        String,
                        String,
                    )>| async move { repo_page(&args, &name, &repo).await }
                }),
            )
            .layer(CorsLayer::new().allow_origin(Any));
    Server::bind(&"[::1]:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
