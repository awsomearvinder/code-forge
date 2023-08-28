use axum::{http::StatusCode, Json};

use crate::Args;

#[derive(serde::Serialize)]
pub(crate) struct CommitLog {
    commits: Vec<Commit>,
}

#[derive(serde::Serialize)]
struct Commit {
    message_header: String,
    message_body: String,
}

pub(crate) async fn repo_page(
    args: &Args,
    entity: &str,
    name: &str,
) -> Result<Json<CommitLog>, StatusCode> {
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