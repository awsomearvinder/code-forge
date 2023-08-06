use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use axum::response::Html;
use axum::{routing, Router, Server};
use clap::Parser;
use git2::BranchType;
use tokio::fs::DirBuilder;
use tokio_stream::StreamExt;

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

async fn repo_page(args: &Args, entity: &str, name: &str) -> Html<String> {
    let repo =
        git2::Repository::open_bare(args.data_dir.join(format!("repositories/{entity}/{name}")))
            .unwrap();
    let mut walk = repo.revwalk().unwrap();
    walk.push(
        repo.find_branch("master", BranchType::Local)
            .unwrap()
            .into_reference()
            .peel_to_commit()
            .unwrap()
            .id(),
    )
    .unwrap();
    let messages: String = walk
        .take(10)
        .map(|oid| {
            let commit = repo.find_commit(oid.unwrap()).unwrap();
            let message = commit.message().unwrap();
            let [header, body @ ..]: &[&str] = &message.split('\n').collect::<Vec<_>>()[..] else { panic!("Should atleast have an empty string in header") };
            format!("<p><b>{}</b></br>{}</p>", header, body.join("</br>"))
        })
        .collect();
    Html(format!("<!DOCTYPE HTML> {}", messages))
}

async fn entity_page(args: &Args, entity_name: &str) -> Html<String> {
    let repo_entry_links: String =
        get_entries(&args.data_dir.join(format!("repositories/{entity_name}")))
            .await
            .into_iter()
            .map(|i| {
                format!(
                    "<a href=\"{entity_name}/{0}\"> {0} </a>\n",
                    i.to_str().unwrap()
                )
            })
            .collect();
    Html(format!("<!DOCTYPE HTML>\n{}", repo_entry_links))
}

async fn home_page(args: &Args) -> Html<String> {
    let repo_entry_links: String = get_entries(&args.data_dir.join("repositories/"))
        .await
        .into_iter()
        .map(|i| format!("<a href=\"{0}\"> {0} </a>\n", i.to_str().unwrap()))
        .collect();
    Html(format!("<!DOCTYPE HTML>\n{}", repo_entry_links))
}

async fn async_main() {
    let args = std::sync::Arc::new(Args::parse());
    datadir_init(&args.data_dir).await;
    let app =
        Router::new()
            .route(
                "/",
                routing::get({
                    let args = args.clone();
                    move || async move { home_page(&args).await }
                }),
            )
            .route(
                "/:entity",
                routing::get({
                    let args = args.clone();
                    move |axum::extract::Path(name): axum::extract::Path<String>| async move {
                        entity_page(&args, &name).await
                    }
                }),
            )
            .route(
                "/:entity/:repo",
                routing::get({
                    let args = args.clone();
                    move |axum::extract::Path((name, repo)): axum::extract::Path<(
                        String,
                        String,
                    )>| async move { repo_page(&args, &name, &repo).await }
                }),
            );
    Server::bind(&"[::1]:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
