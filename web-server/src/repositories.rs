use axum::http::StatusCode;
use git2::Oid;

use crate::Args;

#[derive(serde::Serialize)]
pub(crate) struct CommitLog {
    pub commits: Vec<Commit>,
}

#[derive(serde::Serialize)]
pub(crate) struct Commit {
    message_header: String,
    message_body: String,
    commit_id: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct CommitLogReq {
    #[serde(default)]
    pub(crate) rev: Option<String>,
    #[serde(default)]
    pub(crate) increment: i32,
}

impl CommitLog {
    pub(crate) async fn commit_log(
        args: &Args,
        entity: &str,
        repo_name: &str,
        req: &CommitLogReq,
    ) -> Result<CommitLog, StatusCode> {
        let repo = git2::Repository::open_bare(
            args.data_dir
                .join(format!("repositories/{entity}/{repo_name}")),
        )
        .map_err(|e| match e.code() {
            git2::ErrorCode::NotFound => StatusCode::NOT_FOUND,
            e => {
                panic!("Couldn't open repo {entity}/{repo_name}, got unexpected error: {e:?}")
            }
        })?;
        let mut walk = repo.revwalk().unwrap();

        match &req.rev {
            Some(v) => walk
                .push(
                    repo.find_commit(Oid::from_str(v).unwrap())
                        .map_err(|_| StatusCode::NOT_FOUND)?
                        .id(),
                )
                .unwrap(),
            None => walk
                .push(repo.head().unwrap().peel_to_commit().unwrap().id())
                .unwrap(),
        }

        let messages: Vec<Commit> = walk
            .skip(req.increment.try_into().unwrap())
            .take(10)
            .map(|oid| {
                let commit = repo.find_commit(oid.unwrap()).unwrap();
                let message = commit.message().unwrap_or("(empty commit message)");
                let [header, body @ ..]: &[&str] = &message.split('\n').collect::<Vec<_>>()[..]
                else {
                    unreachable!()
                }; // body is empty in the case where there's no new line
                Commit {
                    message_header: header.to_string(),
                    message_body: body.join("\n"),
                    commit_id: format!("{}", commit.id()),
                }
            })
            .collect();
        Ok(CommitLog { commits: messages })
    }
}
